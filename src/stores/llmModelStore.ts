import { create } from "zustand";
import { produce } from "immer";
import { listen } from "@tauri-apps/api/event";
import { commands, type LlmModelInfo } from "@/bindings";

interface LlmDownloadProgress {
  model_id: string;
  downloaded: number;
  total: number;
  percentage: number;
}

interface LlmDownloadStats {
  startTime: number;
  lastUpdate: number;
  totalDownloaded: number;
  speedMbps: number;
}

interface LlmModelsStore {
  models: LlmModelInfo[];
  activeModelId: string | null;
  downloadProgress: Record<
    string,
    { downloaded: number; total: number; percentage: number }
  >;
  downloadStats: Record<string, { speedMbps: number }>;
  verifyingModels: Record<string, true>;
  isLoading: boolean;

  // Actions
  refresh: () => Promise<void>;
  downloadModel: (modelId: string) => Promise<void>;
  cancelDownload: (modelId: string) => Promise<void>;
  deleteModel: (modelId: string) => Promise<void>;
  setActiveModel: (modelId: string) => Promise<void>;
  importCustom: (path: string) => Promise<{ ok: boolean; error?: string }>;

  // Listener setup (returns unlisten cleanup function)
  initListeners: () => Promise<() => void>;
}

export const useLlmModelStore = create<LlmModelsStore>()((set, get) => ({
  models: [],
  activeModelId: null,
  downloadProgress: {},
  downloadStats: {},
  verifyingModels: {},
  isLoading: false,

  refresh: async () => {
    set({ isLoading: true });
    try {
      const [modelsResult, activeResult] = await Promise.all([
        commands.getLlmModels(),
        commands.getActiveLlmModel(),
      ]);

      if (modelsResult.status === "ok") {
        set({ models: modelsResult.data });
      }
      if (activeResult.status === "ok") {
        set({ activeModelId: activeResult.data });
      }
    } finally {
      set({ isLoading: false });
    }
  },

  downloadModel: async (modelId: string) => {
    set(
      produce((state: LlmModelsStore) => {
        state.downloadProgress[modelId] = {
          downloaded: 0,
          total: 0,
          percentage: 0,
        };
      }),
    );
    await commands.downloadLlmModel(modelId);
  },

  cancelDownload: async (modelId: string) => {
    await commands.cancelLlmDownload(modelId);
    set(
      produce((state: LlmModelsStore) => {
        delete state.downloadProgress[modelId];
        delete state.downloadStats[modelId];
      }),
    );
  },

  deleteModel: async (modelId: string) => {
    await commands.deleteLlmModel(modelId);
    await get().refresh();
  },

  setActiveModel: async (modelId: string) => {
    await commands.setActiveLlmModel(modelId);
    set({ activeModelId: modelId });
  },

  importCustom: async (path: string) => {
    const result = await commands.importCustomLlmModel(path);
    if (result.status === "ok") {
      await get().refresh();
      return { ok: true };
    }
    return { ok: false, error: result.error };
  },

  initListeners: async () => {
    const unlisteners: Array<() => void> = [];

    const unlistenProgress = await listen<LlmDownloadProgress>(
      "llm-download-progress",
      (event) => {
        const progress = event.payload;
        const now = Date.now();

        set(
          produce((state: LlmModelsStore) => {
            state.downloadProgress[progress.model_id] = {
              downloaded: progress.downloaded,
              total: progress.total,
              percentage: progress.percentage,
            };

            const current = state.downloadStats[progress.model_id] as
              | LlmDownloadStats
              | undefined;

            if (!current) {
              (state.downloadStats[progress.model_id] as LlmDownloadStats) = {
                startTime: now,
                lastUpdate: now,
                totalDownloaded: progress.downloaded,
                speedMbps: 0,
              };
            } else {
              const timeDiff = (now - current.lastUpdate) / 1000;
              const bytesDiff = progress.downloaded - current.totalDownloaded;

              if (timeDiff > 0.5) {
                const currentSpeed = bytesDiff / (1024 * 1024) / timeDiff;
                const validSpeed = Math.max(0, currentSpeed);
                const smoothed =
                  current.speedMbps > 0
                    ? current.speedMbps * 0.8 + validSpeed * 0.2
                    : validSpeed;

                (state.downloadStats[progress.model_id] as LlmDownloadStats) = {
                  startTime: current.startTime,
                  lastUpdate: now,
                  totalDownloaded: progress.downloaded,
                  speedMbps: Math.max(0, smoothed),
                };
              }
            }
          }),
        );
      },
    );
    unlisteners.push(unlistenProgress);

    const unlistenVerifyStart = await listen<string>(
      "llm-verification-started",
      (event) => {
        const modelId = event.payload;
        set(
          produce((state: LlmModelsStore) => {
            state.verifyingModels[modelId] = true;
          }),
        );
      },
    );
    unlisteners.push(unlistenVerifyStart);

    const unlistenVerifyEnd = await listen<string>(
      "llm-verification-completed",
      (event) => {
        const modelId = event.payload;
        set(
          produce((state: LlmModelsStore) => {
            delete state.verifyingModels[modelId];
          }),
        );
      },
    );
    unlisteners.push(unlistenVerifyEnd);

    const unlistenComplete = await listen<string>(
      "llm-download-complete",
      (event) => {
        const modelId = event.payload;
        set(
          produce((state: LlmModelsStore) => {
            delete state.downloadProgress[modelId];
            delete state.downloadStats[modelId];
            delete state.verifyingModels[modelId];
          }),
        );
        void (async () => {
          await get().refresh();
          // Auto-activate the freshly downloaded model when nothing is active
          // yet, so the first download is usable without an extra click. Don't
          // steal activation from a model the user already chose.
          if (!get().activeModelId) {
            await get().setActiveModel(modelId);
          }
        })();
      },
    );
    unlisteners.push(unlistenComplete);

    const unlistenCancelled = await listen<string>(
      "llm-download-cancelled",
      (event) => {
        const modelId = event.payload;
        set(
          produce((state: LlmModelsStore) => {
            delete state.downloadProgress[modelId];
            delete state.downloadStats[modelId];
          }),
        );
        void get().refresh();
      },
    );
    unlisteners.push(unlistenCancelled);

    const unlistenFailed = await listen<{ model_id: string; error: string }>(
      "llm-download-failed",
      (event) => {
        const modelId = event.payload.model_id;
        set(
          produce((state: LlmModelsStore) => {
            delete state.downloadProgress[modelId];
            delete state.downloadStats[modelId];
            delete state.verifyingModels[modelId];
          }),
        );
        void get().refresh();
      },
    );
    unlisteners.push(unlistenFailed);

    const unlistenDeleted = await listen("llm-deleted", () => {
      void get().refresh();
    });
    unlisteners.push(unlistenDeleted);

    const unlistenImported = await listen("llm-custom-model-imported", () => {
      void get().refresh();
    });
    unlisteners.push(unlistenImported);

    const unlistenLoaded = await listen("llm-model-loaded", () => {
      void get().refresh();
    });
    unlisteners.push(unlistenLoaded);

    const unlistenUnloaded = await listen("llm-model-unloaded", () => {
      void get().refresh();
    });
    unlisteners.push(unlistenUnloaded);

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  },
}));
