import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { Check, Copy, FileAudio, History, Upload, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  commands,
  events,
  type AudioFileDetails,
  type FileTranscriptionError,
  type FileTranscriptionResult,
  type FileTranscriptionStage,
} from "@/bindings";
import { Alert } from "@/components/ui/Alert";
import { Button } from "@/components/ui/Button";
import { useSettings } from "@/hooks/useSettings";
import { LANGUAGES } from "@/lib/constants/languages";
import { formatDuration, formatFileSize } from "@/lib/utils/format";
import { useModelStore } from "@/stores/modelStore";
import { useNavigationStore } from "@/stores/navigationStore";

/** Everything the workspace can be doing at a given moment. */
type Phase =
  | { status: "idle" }
  | { status: "inspecting" }
  | {
      status: "running";
      file: AudioFileDetails;
      stage: FileTranscriptionStage;
      progress: number | null;
      cancelling: boolean;
    }
  | { status: "done"; file: AudioFileDetails; result: FileTranscriptionResult }
  | {
      status: "failed";
      file: AudioFileDetails | null;
      error: FileTranscriptionError;
    };

const STAGE_LABEL_KEYS: Record<FileTranscriptionStage, string> = {
  decoding: "fileTranscription.stages.decoding",
  loading_model: "fileTranscription.stages.loadingModel",
  transcribing: "fileTranscription.stages.transcribing",
  saving: "fileTranscription.stages.saving",
};

const ERROR_MESSAGE_KEYS: Record<FileTranscriptionError["kind"], string> = {
  unsupported_format: "fileTranscription.errors.unsupportedFormat",
  unsupported_codec: "fileTranscription.errors.unsupportedCodec",
  unreadable_file: "fileTranscription.errors.unreadableFile",
  corrupt_audio: "fileTranscription.errors.corruptAudio",
  empty_audio: "fileTranscription.errors.emptyAudio",
  file_too_long: "fileTranscription.errors.fileTooLong",
  no_speech: "fileTranscription.errors.noSpeech",
  model_unavailable: "fileTranscription.errors.modelUnavailable",
  transcription_failed: "fileTranscription.errors.transcriptionFailed",
  storage_failed: "fileTranscription.errors.storageFailed",
  busy: "fileTranscription.errors.busy",
  cancelled: "fileTranscription.errors.cancelled",
};

/**
 * Turn a rejected command into a `FileTranscriptionError`.
 *
 * Typed errors come back as `{ kind, detail }`; anything else is a transport or
 * runtime failure that never reached our error enum, so it becomes a generic
 * transcription failure rather than a raw string on screen.
 */
const toTranscriptionError = (error: unknown): FileTranscriptionError => {
  if (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    typeof (error as { kind: unknown }).kind === "string"
  ) {
    return error as FileTranscriptionError;
  }
  return { kind: "transcription_failed", detail: String(error) };
};

export const FileTranscription: React.FC = () => {
  const { t } = useTranslation();
  const { settings } = useSettings();
  const setSection = useNavigationStore((state) => state.setSection);
  const models = useModelStore((state) => state.models);
  const initializeModels = useModelStore((state) => state.initialize);

  const [phase, setPhase] = useState<Phase>({ status: "idle" });
  const [isDragActive, setIsDragActive] = useState(false);
  const [showCopied, setShowCopied] = useState(false);
  const [dropRejected, setDropRejected] = useState(false);
  const [extensions, setExtensions] = useState<string[]>([]);

  // The drag/drop listener and the running job both outlive individual
  // renders, so callbacks read the current phase through a ref.
  const phaseRef = useRef<Phase>(phase);
  useEffect(() => {
    phaseRef.current = phase;
  }, [phase]);

  useEffect(() => {
    void initializeModels();
  }, [initializeModels]);

  // The "already running" notice is an acknowledgement of a gesture, not a
  // state, so it clears itself rather than needing to be dismissed.
  useEffect(() => {
    if (!dropRejected) return;
    const timer = setTimeout(() => setDropRejected(false), 4000);
    return () => clearTimeout(timer);
  }, [dropRejected]);

  useEffect(() => {
    let cancelled = false;
    void commands.supportedAudioExtensions().then((list) => {
      if (!cancelled) setExtensions(list);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const startTranscription = useCallback(async (file: AudioFileDetails) => {
    setPhase({
      status: "running",
      file,
      stage: "decoding",
      progress: 0,
      cancelling: false,
    });

    try {
      const result = await commands.transcribeAudioFile(file.path);
      if (result.status === "ok") {
        setPhase({ status: "done", file, result: result.data });
      } else {
        setPhase({
          status: "failed",
          file,
          error: toTranscriptionError(result.error),
        });
      }
    } catch (error) {
      setPhase({ status: "failed", file, error: toTranscriptionError(error) });
    }
  }, []);

  // Picking a file is the instruction — on a page called Transcribe File,
  // dropping an audio file has already said what to do, so there is no second
  // confirmation. Everything the old confirmation step showed (name, duration,
  // size, model, language) stays on screen for the whole run instead, next to
  // Cancel, so a wrong model or language is still caught in the first seconds.
  const selectFile = useCallback(
    async (path: string) => {
      setPhase({ status: "inspecting" });
      try {
        const result = await commands.inspectAudioFile(path);
        if (result.status === "ok") {
          // Probing already refused anything undecodable, so this cannot start a
          // job that was never going to finish.
          await startTranscription(result.data);
        } else {
          setPhase({
            status: "failed",
            file: null,
            error: toTranscriptionError(result.error),
          });
        }
      } catch (error) {
        setPhase({
          status: "failed",
          file: null,
          error: toTranscriptionError(error),
        });
      }
    },
    [startTranscription],
  );

  // Registered only while this page is mounted, so dropping a file on any
  // other page keeps doing whatever that page does.
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let disposed = false;

    const setup = async () => {
      const webview = getCurrentWebview();
      const stop = await webview.onDragDropEvent((event) => {
        if (event.payload.type === "enter") {
          setIsDragActive(true);
        } else if (event.payload.type === "leave") {
          setIsDragActive(false);
        } else if (event.payload.type === "drop") {
          setIsDragActive(false);
          // One job owns the engine at a time. A second drop can't queue, and
          // silently swallowing it would look like the app missed the gesture,
          // so say why nothing happened. This also closes the double-drop race:
          // the guard covers `inspecting` too, not just `running`.
          const status = phaseRef.current.status;
          if (status === "running" || status === "inspecting") {
            setDropRejected(true);
            return;
          }
          const paths = event.payload.paths;
          if (paths && paths.length > 0) {
            void selectFile(paths[0]);
          }
        }
      });
      if (disposed) {
        stop();
      } else {
        unlisten = stop;
      }
    };

    void setup();

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [selectFile]);

  // Stage/progress updates for the job this page started.
  useEffect(() => {
    const unlisten = events.fileTranscriptionProgress.listen((event) => {
      setPhase((current) =>
        current.status === "running"
          ? {
              ...current,
              stage: event.payload.stage,
              progress: event.payload.progress,
            }
          : current,
      );
    });
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  const openFilePicker = async () => {
    const selected = await open({
      multiple: false,
      filters: extensions.length
        ? [{ name: t("fileTranscription.pickerFilter"), extensions }]
        : undefined,
    });
    if (selected && typeof selected === "string") {
      await selectFile(selected);
    }
  };

  const requestCancel = async () => {
    setPhase((current) =>
      current.status === "running" ? { ...current, cancelling: true } : current,
    );
    await commands.cancelFileTranscription();
  };

  const copyTranscript = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setShowCopied(true);
      setTimeout(() => setShowCopied(false), 2000);
    } catch (error) {
      console.error("Failed to copy transcript:", error);
    }
  };

  const reset = () => {
    setShowCopied(false);
    setPhase({ status: "idle" });
  };

  const modelName = useMemo(() => {
    const id = settings?.selected_model ?? "";
    if (!id) return t("fileTranscription.file.noModel");
    return models.find((model) => model.id === id)?.name ?? id;
  }, [models, settings?.selected_model, t]);

  const languageName = useMemo(() => {
    const value = settings?.selected_language ?? "auto";
    if (value === "auto") return t("settings.general.language.auto");
    return (
      LANGUAGES.find((language) => language.value === value)?.label ?? value
    );
  }, [settings?.selected_language, t]);

  const supportedFormats = useMemo(
    () => extensions.join(", ").toUpperCase(),
    [extensions],
  );

  // Pulled out of the JSX so the retry button keeps the narrowed type: a
  // callback defined inside the render can't rely on narrowing of `phase`.
  const failedFile = phase.status === "failed" ? phase.file : null;

  // The empty state — nothing picked yet, or a pick that never produced a file.
  // The drop zone is the whole point of the page here, so it takes the full
  // height; every other state is a card whose height follows its content.
  const showsDropZone =
    phase.status === "idle" ||
    phase.status === "inspecting" ||
    (phase.status === "failed" && phase.file === null);

  // Only the unsupported-codec case interpolates anything: its detail is the
  // codec name, which the message needs so the user knows it is Opus (or
  // whatever) rather than something wrong with their file.
  const errorMessage =
    phase.status === "failed"
      ? t(
          ERROR_MESSAGE_KEYS[phase.error.kind],
          phase.error.kind === "unsupported_codec"
            ? { codec: phase.error.detail }
            : undefined,
        )
      : null;

  return (
    // Same max-w-3xl column as every other page — the width already matches the
    // rest of the app; it is the vertical emptiness underneath that was the
    // problem, so only the height is claimed here.
    <div className="max-w-3xl w-full mx-auto flex-1 flex flex-col gap-2">
      <div className="px-4">
        <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
          {t("fileTranscription.title")}
        </h2>
        <p className="text-sm text-text/60 mt-1">
          {t("fileTranscription.subtitle")}
        </p>
      </div>

      <div
        className={`bg-background border border-mid-gray/20 rounded-lg p-4 space-y-4 flex flex-col ${
          showsDropZone ? "flex-1" : ""
        }`}
      >
        {showsDropZone && (
          <>
            <DropZone
              isDragActive={isDragActive}
              isBusy={phase.status === "inspecting"}
              supportedFormats={supportedFormats}
              onClick={() => void openFilePicker()}
            />
            {/* The privacy line lived on the confirmation card that auto-start
                removed. It belongs here anyway: it matters while someone is
                deciding whether to hand over a file, not after they have. */}
            <p className="text-xs text-text/50 text-center">
              {t("fileTranscription.privacyNote")}
            </p>
          </>
        )}

        {phase.status === "failed" && (
          <div className="space-y-3">
            <Alert variant="error">{errorMessage}</Alert>
            {failedFile !== null && (
              <div className="flex gap-2">
                <Button
                  variant="primary"
                  size="md"
                  onClick={() => void startTranscription(failedFile)}
                >
                  {t("fileTranscription.actions.retry")}
                </Button>
                <Button variant="secondary" size="md" onClick={reset}>
                  {t("fileTranscription.actions.chooseAnother")}
                </Button>
              </div>
            )}
          </div>
        )}

        {phase.status === "running" && (
          <div className="space-y-4">
            <FileSummary
              file={phase.file}
              modelName={modelName}
              languageName={languageName}
            />
            <ProgressPanel
              stageLabel={t(STAGE_LABEL_KEYS[phase.stage])}
              progress={phase.progress}
              cancelling={phase.cancelling}
            />
            <div className="flex items-center gap-3 flex-wrap">
              <Button
                variant="secondary"
                size="md"
                disabled={phase.cancelling}
                onClick={() => void requestCancel()}
                className="flex items-center gap-2"
              >
                <X className="w-4 h-4" />
                <span>
                  {phase.cancelling
                    ? t("fileTranscription.stages.cancelling")
                    : t("fileTranscription.actions.cancel")}
                </span>
              </Button>
              {dropRejected && (
                <p className="text-xs text-text/60">
                  {t("fileTranscription.alreadyRunning")}
                </p>
              )}
            </div>
          </div>
        )}

        {phase.status === "done" && (
          <div className="space-y-4">
            <FileSummary
              file={phase.file}
              modelName={modelName}
              languageName={languageName}
            />

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <h3 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
                  {t("fileTranscription.result.title")}
                </h3>
                <div className="flex gap-2">
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={() => void copyTranscript(phase.result.text)}
                    className="flex items-center gap-2"
                  >
                    {showCopied ? (
                      <Check className="w-4 h-4" />
                    ) : (
                      <Copy className="w-4 h-4" />
                    )}
                    <span>
                      {showCopied
                        ? t("fileTranscription.actions.copied")
                        : t("fileTranscription.actions.copy")}
                    </span>
                  </Button>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => setSection("history")}
                    className="flex items-center gap-2"
                  >
                    <History className="w-4 h-4" />
                    <span>{t("fileTranscription.actions.openInHistory")}</span>
                  </Button>
                </div>
              </div>

              <p className="text-sm text-text/90 select-text cursor-text whitespace-pre-wrap break-words bg-mid-gray/5 border border-mid-gray/20 rounded-lg p-3">
                {phase.result.text}
              </p>
              <p className="text-xs text-text/50">
                {t("fileTranscription.result.savedToHistory")}
              </p>
            </div>

            <Button variant="secondary" size="md" onClick={reset}>
              {t("fileTranscription.actions.chooseAnother")}
            </Button>
          </div>
        )}
      </div>
    </div>
  );
};

interface DropZoneProps {
  isDragActive: boolean;
  isBusy: boolean;
  supportedFormats: string;
  onClick: () => void;
}

const DropZone: React.FC<DropZoneProps> = ({
  isDragActive,
  isBusy,
  supportedFormats,
  onClick,
}) => {
  const { t } = useTranslation();

  return (
    <button
      type="button"
      onClick={onClick}
      disabled={isBusy}
      aria-label={t("fileTranscription.dropZone.label")}
      // flex-1 fills the panel, which fills the page. min-h-[14rem] keeps the
      // target usable if the window is short; below that the page scrolls
      // rather than squashing the label, so there is never a second scrollbar.
      className={`w-full flex-1 min-h-[14rem] border border-dashed rounded-lg flex flex-col items-center justify-center gap-2 px-6 py-12 transition-colors cursor-pointer disabled:cursor-wait focus:outline-none focus:ring-2 focus:ring-logo-primary ${
        isDragActive
          ? "border-logo-primary/60 bg-logo-primary/5"
          : "border-mid-gray/40 hover:border-logo-primary/60 hover:bg-logo-primary/5"
      }`}
    >
      {isDragActive ? (
        <Upload className="w-10 h-10 text-logo-primary" />
      ) : (
        <FileAudio className="w-10 h-10 text-mid-gray" />
      )}
      <span className="text-sm font-medium">
        {isBusy
          ? t("fileTranscription.dropZone.reading")
          : t("fileTranscription.dropZone.label")}
      </span>
      <span className="text-xs text-mid-gray">
        {t("fileTranscription.dropZone.sublabel")}
      </span>
      {supportedFormats.length > 0 && (
        <span className="text-xs text-text/40">
          {t("fileTranscription.supportedFormats", {
            formats: supportedFormats,
          })}
        </span>
      )}
    </button>
  );
};

interface FileSummaryProps {
  file: AudioFileDetails;
  modelName: string;
  languageName: string;
}

const FileSummary: React.FC<FileSummaryProps> = ({
  file,
  modelName,
  languageName,
}) => {
  const { t } = useTranslation();
  const duration = formatDuration(file.duration_ms);

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <FileAudio className="w-4 h-4 shrink-0 text-mid-gray" />
        <p className="text-sm font-medium truncate" title={file.file_name}>
          {file.file_name}
        </p>
      </div>
      <dl className="grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-text/60">
        <SummaryRow
          label={t("fileTranscription.file.duration")}
          value={duration ?? t("fileTranscription.file.unknownDuration")}
        />
        <SummaryRow
          label={t("fileTranscription.file.size")}
          value={formatFileSize(file.size_bytes)}
        />
        <SummaryRow
          label={t("fileTranscription.file.model")}
          value={modelName}
        />
        <SummaryRow
          label={t("fileTranscription.file.language")}
          value={languageName}
        />
      </dl>
    </div>
  );
};

const SummaryRow: React.FC<{ label: string; value: string }> = ({
  label,
  value,
}) => (
  <div className="flex gap-2 min-w-0">
    <dt className="shrink-0">{label}</dt>
    <dd className="truncate text-text/90" title={value}>
      {value}
    </dd>
  </div>
);

interface ProgressPanelProps {
  stageLabel: string;
  progress: number | null;
  cancelling: boolean;
}

/**
 * Determinate only when the backend actually knows the fraction. Inference is
 * a single opaque call, so those stages get a moving bar with no number rather
 * than a percentage we'd be making up.
 */
const ProgressPanel: React.FC<ProgressPanelProps> = ({
  stageLabel,
  progress,
  cancelling,
}) => {
  const { t } = useTranslation();
  const percent =
    progress === null
      ? null
      : Math.round(Math.min(Math.max(progress, 0), 1) * 100);

  return (
    <div className="space-y-2">
      <style>{`
        @keyframes file-transcription-indeterminate {
          0% { transform: translateX(-100%); }
          100% { transform: translateX(400%); }
        }
      `}</style>
      <div className="flex items-center justify-between text-xs">
        <span className="text-text/80">
          {cancelling ? t("fileTranscription.stages.cancelling") : stageLabel}
        </span>
        {percent !== null && (
          <span className="text-text/60 tabular-nums">
            {t("fileTranscription.percent", { percent })}
          </span>
        )}
      </div>
      <div
        className="h-1.5 w-full rounded-full bg-mid-gray/20 overflow-hidden"
        role="progressbar"
        aria-label={stageLabel}
        aria-valuenow={percent ?? undefined}
        aria-valuemin={percent === null ? undefined : 0}
        aria-valuemax={percent === null ? undefined : 100}
      >
        {percent === null ? (
          <div
            className="h-full w-1/4 rounded-full bg-logo-primary"
            style={{
              animation:
                "file-transcription-indeterminate 1.4s ease-in-out infinite",
            }}
          />
        ) : (
          <div
            className="h-full rounded-full bg-logo-primary transition-[width] duration-200"
            style={{ width: `${percent}%` }}
          />
        )}
      </div>
      {cancelling && (
        <p className="text-xs text-text/50">
          {t("fileTranscription.cancelPending")}
        </p>
      )}
    </div>
  );
};
