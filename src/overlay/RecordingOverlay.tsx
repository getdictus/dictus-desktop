import { listen } from "@tauri-apps/api/event";
import React, { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { CancelIcon } from "../components/icons";
import "./RecordingOverlay.css";
import { commands } from "@/bindings";
import i18n, { syncLanguageFromSettings } from "@/i18n";
import { getLanguageDirection } from "@/lib/utils/rtl";

type OverlayState = "recording" | "transcribing" | "processing";

const BAR_COUNT = 30;

function interpolateLevels(source: number[], barCount: number): number[] {
  if (source.length === 0) return Array(barCount).fill(0);

  // Compute overall energy (weighted toward low freq = voice)
  let weightedSum = 0;
  let weightTotal = 0;
  for (let i = 0; i < source.length; i++) {
    const weight = 1.0 - (i / source.length) * 0.5;
    weightedSum += source[i] * source[i] * weight;
    weightTotal += weight;
  }
  const energy = Math.min(Math.sqrt(weightedSum / weightTotal) * 8.0, 1.0);

  // Per-bar random multiplier for organic variation (different each bar, each frame)
  const center = (barCount - 1) / 2;
  const result = Array.from({ length: barCount }, (_, i) => {
    const distFromCenter = Math.abs(i - center) / center;
    // Envelope: 2/3 center = full, edges drop off
    const envelope =
      distFromCenter < 0.65
        ? 1.0
        : 1.0 - ((distFromCenter - 0.65) / 0.35) * 0.8;
    // Random variation per bar: 0.5 to 1.0 — makes each bar different height
    const randomFactor = 0.5 + Math.random() * 0.5;
    const value = energy * envelope * randomFactor;
    return value < 0.02 ? 0 : Math.min(value, 1);
  });

  return result;
}

function tickLevels(current: number[], targets: number[]): number[] {
  return current.map((prev, i) => {
    const target = targets[i] ?? 0;
    let next: number;
    if (target > prev) {
      next = prev + (target - prev) * 0.7; // fast attack = minimal perceived delay
    } else {
      next = target + (prev - target) * 0.85;
    }
    return next < 0.005 ? 0 : next;
  });
}

function transcribingEnergy(
  index: number,
  barCount: number,
  phase: number,
): number {
  const normalizedIndex = index / Math.max(barCount - 1, 1);
  const sineValue = Math.sin(2 * Math.PI * (normalizedIndex + phase));
  return 0.2 + 0.25 * (sineValue + 1.0);
}

function cylonPeakAtPosition(barCount: number, peakPos: number): number[] {
  const halfWidth = 6.0;
  return Array.from({ length: barCount }, (_, i) => {
    const dist = Math.abs(i - peakPos);
    if (dist >= halfWidth) return 0.05;
    // Cosine bump: 1.0 at the peak (dist=0), smoothly decaying to 0 at the
    // halfWidth edges. Replaces the previous triangular falloff + 5-palier
    // quantization — gives a rounded, sinusoid-like wave that matches the
    // smooth language of the transcribing state.
    const t = dist / halfWidth;
    return Math.max(0.05, 0.5 + 0.5 * Math.cos(Math.PI * t));
  });
}

function cylonPeak(barCount: number, phase: number): number[] {
  const peakPos = ((barCount - 1) * (1 + Math.sin(2 * Math.PI * phase))) / 2;
  return cylonPeakAtPosition(barCount, peakPos);
}

function easeInOutCubic(t: number): number {
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

const PROCESSING_PHASE_RATE = 0.7; // Hz
const OUTRO_CENTER_MS = 300;
const OUTRO_COLLAPSE_MS = 400;
const OUTRO_TOTAL_MS = OUTRO_CENTER_MS + OUTRO_COLLAPSE_MS;

function getBarColor(index: number, barCount: number): string {
  const center = (barCount - 1) / 2;
  const distanceFromCenter = Math.abs(index - center) / center;
  if (distanceFromCenter < 0.4) {
    return "#6BA3FF";
  }
  const opacity = (1.0 - distanceFromCenter) * 0.9 + 0.15;
  return `rgba(255,255,255,${opacity.toFixed(2)})`;
}

const RecordingOverlay: React.FC = () => {
  const { t } = useTranslation();
  const [isVisible, setIsVisible] = useState(false);
  const [state, setState] = useState<OverlayState>("recording");
  const [levels, setLevels] = useState<number[]>(Array(BAR_COUNT).fill(0));
  const [reducedMotion, setReducedMotion] = useState(
    () => window.matchMedia("(prefers-reduced-motion: reduce)").matches,
  );
  const smoothedLevelsRef = useRef<number[]>(Array(BAR_COUNT).fill(0));
  const targetLevelsRef = useRef<number[]>(Array(BAR_COUNT).fill(0));
  const phaseRef = useRef<number>(0);
  const rafIdRef = useRef<number>(0);
  const lastTimeRef = useRef<number>(0);
  const stateRef = useRef<OverlayState>("recording");
  const reducedMotionRef = useRef<boolean>(false);
  const outroStartTimeRef = useRef<number | null>(null);
  const direction = getLanguageDirection(i18n.language);

  useEffect(() => {
    stateRef.current = state;
  }, [state]);

  useEffect(() => {
    reducedMotionRef.current = reducedMotion;
  }, [reducedMotion]);

  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    const handler = (e: MediaQueryListEvent) => setReducedMotion(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);

  useEffect(() => {
    // Track cancellation so a StrictMode cleanup that fires before the async
    // setup finishes still tears down whichever listeners did register.
    let cancelled = false;
    let unlistenShow: (() => void) | undefined;
    let unlistenHide: (() => void) | undefined;
    let unlistenLevel: (() => void) | undefined;

    const setupEventListeners = async () => {
      const [us, uh, ul] = await Promise.all([
        listen("show-overlay", async (event) => {
          await syncLanguageFromSettings();
          const overlayState = event.payload as OverlayState;
          setState(overlayState);
          phaseRef.current = 0;
          smoothedLevelsRef.current = Array(BAR_COUNT).fill(0);
          targetLevelsRef.current = Array(BAR_COUNT).fill(0);
          outroStartTimeRef.current = null;
          setIsVisible(true);
        }),
        listen("hide-overlay", () => {
          // Defensive idempotency: if an outro is already in flight (e.g.
          // duplicate-listener invocation under StrictMode), bail.
          if (outroStartTimeRef.current !== null) return;

          if (stateRef.current === "processing" && !reducedMotionRef.current) {
            // Outro: let the cylon keep advancing while the bar pattern blends
            // toward a centered shape; then collapse heights to zero; then hide.
            // No position/velocity capture needed — the hybrid blend in the
            // animate loop reads phaseRef as it advances naturally.
            outroStartTimeRef.current = performance.now();
            window.setTimeout(() => {
              outroStartTimeRef.current = null;
              setIsVisible(false);
            }, OUTRO_TOTAL_MS);
          } else {
            cancelAnimationFrame(rafIdRef.current);
            setIsVisible(false);
          }
        }),
        listen<number[]>("mic-level", (event) => {
          const raw = event.payload as number[];
          targetLevelsRef.current = interpolateLevels(raw, BAR_COUNT);
        }),
      ]);

      if (cancelled) {
        us();
        uh();
        ul();
      } else {
        unlistenShow = us;
        unlistenHide = uh;
        unlistenLevel = ul;
      }
    };

    setupEventListeners();

    return () => {
      cancelled = true;
      unlistenShow?.();
      unlistenHide?.();
      unlistenLevel?.();
    };
  }, []);

  useEffect(() => {
    if (!isVisible) return;

    const animate = (timestamp: number) => {
      const dt = lastTimeRef.current
        ? (timestamp - lastTimeRef.current) / 1000
        : 1 / 60;
      lastTimeRef.current = timestamp;

      let targets: number[];

      if (state === "processing" && outroStartTimeRef.current !== null) {
        const elapsed = timestamp - outroStartTimeRef.current;
        const centerPos = (BAR_COUNT - 1) / 2;
        if (elapsed < OUTRO_CENTER_MS) {
          // Outro phase 1: hybrid blend — cylon keeps advancing naturally
          // (velocity continuous at handoff), while the rendered peak
          // position blends toward the centered shape via easeInOutCubic.
          phaseRef.current += dt * PROCESSING_PHASE_RATE;
          const naturalPeakPos =
            ((BAR_COUNT - 1) * (1 + Math.sin(2 * Math.PI * phaseRef.current))) /
            2;
          const t = elapsed / OUTRO_CENTER_MS;
          const eased = easeInOutCubic(t);
          const peakPos = naturalPeakPos * (1 - eased) + centerPos * eased;
          targets = cylonPeakAtPosition(BAR_COUNT, peakPos);
        } else {
          // Outro phase 2: collapse the centered peak smoothly to zero
          const collapseT = Math.min(
            1,
            (elapsed - OUTRO_CENTER_MS) / OUTRO_COLLAPSE_MS,
          );
          const eased = easeInOutCubic(collapseT);
          const scale = 1 - eased;
          const base = cylonPeakAtPosition(BAR_COUNT, centerPos);
          targets = base.map((v) => v * scale);
        }
        // Outro-only: bar heights glide toward quantized targets via attack/
        // release lerp instead of snapping. Cubique palier targets stay, but
        // when the cylon's slow-velocity moments would otherwise leave bar
        // heights frozen for several frames, the lerp keeps them moving.
        smoothedLevelsRef.current = tickLevels(
          smoothedLevelsRef.current,
          targets,
        );
        setLevels([...smoothedLevelsRef.current]);
        rafIdRef.current = requestAnimationFrame(animate);
        return;
      }

      if (state === "recording") {
        targets = targetLevelsRef.current;
      } else if (state === "transcribing") {
        // Sine wave sweep, 1 cycle/sec
        phaseRef.current += dt * 1.0;
        targets = Array.from({ length: BAR_COUNT }, (_, i) =>
          transcribingEnergy(i, BAR_COUNT, phaseRef.current),
        );
      } else {
        // processing: traveling peak, PROCESSING_PHASE_RATE Hz, 5-level quantization
        phaseRef.current += dt * PROCESSING_PHASE_RATE;
        targets = cylonPeak(BAR_COUNT, phaseRef.current);
      }

      if (state === "processing") {
        // Direct assignment: smoothing would erase the cubique grain
        smoothedLevelsRef.current = targets;
      } else {
        smoothedLevelsRef.current = tickLevels(
          smoothedLevelsRef.current,
          targets,
        );
      }
      setLevels([...smoothedLevelsRef.current]);
      rafIdRef.current = requestAnimationFrame(animate);
    };

    rafIdRef.current = requestAnimationFrame(animate);
    return () => {
      cancelAnimationFrame(rafIdRef.current);
      lastTimeRef.current = 0;
    };
  }, [isVisible, state]);

  return (
    <div dir={direction} style={{ display: "flex", alignItems: "center" }}>
      <div className={`recording-overlay ${isVisible ? "fade-in" : ""}`}>
        <div className="overlay-middle">
          {(state === "recording" ||
            state === "transcribing" ||
            (state === "processing" && !reducedMotion)) && (
            <div className="bars-container">
              {levels.map((v, i) => (
                <div
                  key={i}
                  className="bar"
                  style={{
                    height: `${Math.max(2, v * 64)}px`,
                    backgroundColor: getBarColor(i, BAR_COUNT),
                  }}
                />
              ))}
            </div>
          )}
          {state === "processing" && reducedMotion && (
            <div className="transcribing-text">{t("overlay.processing")}</div>
          )}
        </div>
      </div>

      {state === "recording" && (
        <div
          className="cancel-pill"
          onClick={() => {
            commands.cancelOperation();
          }}
        >
          <CancelIcon width={38} height={38} />
        </div>
      )}
    </div>
  );
};

export default RecordingOverlay;
