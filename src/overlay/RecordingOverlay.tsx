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
  const halfWidth = 10.0;
  // Crest matches transcribing's crest (0.7) so the transition reads as the
  // same visual family. Baseline is much lower (close to the recording
  // waveform's silent floor) because the cylon is a localized peak, not a
  // continuous wave — bars far from the peak should genuinely drop, not sit
  // at the transcribing sine's wave-trough height.
  const baseline = 0.05;
  const crest = 0.7;
  return Array.from({ length: barCount }, (_, i) => {
    const dist = Math.abs(i - peakPos);
    if (dist >= halfWidth) return baseline;
    const t = dist / halfWidth;
    const bump = 0.5 + 0.5 * Math.cos(Math.PI * t); // 0..1
    return baseline + (crest - baseline) * bump;
  });
}

function cylonPeak(barCount: number, phase: number): number[] {
  const peakPos = ((barCount - 1) * (1 + Math.sin(2 * Math.PI * phase))) / 2;
  return cylonPeakAtPosition(barCount, peakPos);
}

function easeInOutCubic(t: number): number {
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

// Cubic Hermite spline from (startPos, startVelocity) to (endPos, velocity=0).
// startVelocity is in position-units per normalized-time-unit (multiply real-
// time velocity by the duration in seconds to get the correct scaling).
function hermiteToTarget(
  startPos: number,
  startVelocity: number,
  endPos: number,
  t: number,
): number {
  const t2 = t * t;
  const t3 = t2 * t;
  const h00 = 2 * t3 - 3 * t2 + 1;
  const h10 = t3 - 2 * t2 + t;
  const h01 = -2 * t3 + 3 * t2;
  return h00 * startPos + h10 * startVelocity + h01 * endPos;
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
  const outroStartPeakPosRef = useRef<number>(0);
  const outroStartVelocityRef = useRef<number>(0);
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
            // Outro: capture the cylon's current peak position AND its
            // instantaneous velocity, then drive Phase 1 via a velocity-
            // matched Hermite spline. Velocity continuity at the handoff
            // eliminates the "abrupt stop" Pierre still perceives with the
            // zero-derivative easeInOutCubic curve.
            //
            // Cap the velocity at 3 × distance-to-center in normalized time
            // units — that's the largest velocity for which the Hermite
            // remains monotonic. Above 3D the spline overshoots and
            // U-turns, which reintroduces the perceived freeze at the
            // U-turn point.
            //
            // If the cylon is moving away from center, we use velocity 0
            // (any non-zero velocity in that direction guarantees a
            // U-turn) — same visual as the previous purely-eased Phase 1
            // for those cases, kept as a fallback.
            const currentPhase = phaseRef.current;
            const currentPeakPos =
              ((BAR_COUNT - 1) * (1 + Math.sin(2 * Math.PI * currentPhase))) /
              2;
            const centerPosCapture = (BAR_COUNT - 1) / 2;
            const directionToCenter = centerPosCapture - currentPeakPos;
            const distanceToCenter = Math.abs(directionToCenter);
            const cylonVelocity =
              (BAR_COUNT - 1) *
              Math.PI *
              PROCESSING_PHASE_RATE *
              Math.cos(2 * Math.PI * currentPhase);
            const movingTowardCenter =
              cylonVelocity * directionToCenter > 0 && distanceToCenter > 0.01;
            let outroStartVelocity = 0;
            if (movingTowardCenter) {
              const cylonV_normalized =
                cylonVelocity * (OUTRO_CENTER_MS / 1000);
              const monotoneCap = 3 * distanceToCenter;
              outroStartVelocity =
                Math.sign(cylonV_normalized) *
                Math.min(Math.abs(cylonV_normalized), monotoneCap);
            }
            outroStartPeakPosRef.current = currentPeakPos;
            outroStartVelocityRef.current = outroStartVelocity;
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
          // Outro phase 1: Hermite spline from (captured peak position with
          // matched-and-capped cylon velocity) to (center with velocity 0).
          // The captured starting velocity matches the cylon's instantaneous
          // velocity at hide-time (when toward center, capped at 3*distance
          // for monotonicity), so no perceptible velocity discontinuity at
          // the handoff and no U-turn mid-trajectory.
          const t = elapsed / OUTRO_CENTER_MS;
          const peakPos = hermiteToTarget(
            outroStartPeakPosRef.current,
            outroStartVelocityRef.current,
            centerPos,
            t,
          );
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
        // Direct assignment, same as the processing path. tickLevels here
        // was originally added to soften palier-stepping under the old
        // cubique quantization; with continuous cosine heights it only
        // introduces lag — specifically its release factor (0.85) makes
        // shrinking bars linger for ~10 frames behind their targets, which
        // reads as the "stop and resume" Pierre keeps perceiving at the
        // cylon → outro handoff.
        smoothedLevelsRef.current = targets;
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
