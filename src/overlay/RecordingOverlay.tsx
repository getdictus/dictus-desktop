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

function processingEnergy(
  index: number,
  barCount: number,
  phase: number,
): number {
  const normalizedIndex = index / Math.max(barCount - 1, 1);
  const sineValue = Math.sin(2 * Math.PI * (normalizedIndex + phase));
  return 0.2 + 0.25 * (sineValue + 1.0);
}

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
  const smoothedLevelsRef = useRef<number[]>(Array(BAR_COUNT).fill(0));
  const targetLevelsRef = useRef<number[]>(Array(BAR_COUNT).fill(0));
  const phaseRef = useRef<number>(0);
  const rafIdRef = useRef<number>(0);
  const lastTimeRef = useRef<number>(0);
  const direction = getLanguageDirection(i18n.language);

  useEffect(() => {
    const setupEventListeners = async () => {
      const unlistenShow = await listen("show-overlay", async (event) => {
        await syncLanguageFromSettings();
        const overlayState = event.payload as OverlayState;
        setState(overlayState);
        phaseRef.current = 0;
        smoothedLevelsRef.current = Array(BAR_COUNT).fill(0);
        targetLevelsRef.current = Array(BAR_COUNT).fill(0);
        setIsVisible(true);
      });

      const unlistenHide = await listen("hide-overlay", () => {
        cancelAnimationFrame(rafIdRef.current);
        setIsVisible(false);
      });

      const unlistenLevel = await listen<number[]>("mic-level", (event) => {
        const raw = event.payload as number[];
        targetLevelsRef.current = interpolateLevels(raw, BAR_COUNT);
      });

      return () => {
        unlistenShow();
        unlistenHide();
        unlistenLevel();
      };
    };

    setupEventListeners();
  }, []);

  useEffect(() => {
    if (!isVisible) return;

    const animate = (timestamp: number) => {
      const dt = lastTimeRef.current
        ? (timestamp - lastTimeRef.current) / 1000
        : 1 / 60;
      lastTimeRef.current = timestamp;

      let targets: number[];
      if (state === "recording") {
        targets = targetLevelsRef.current;
      } else {
        // Sine wave at 2x iOS speed (1 cycle/sec instead of 0.5)
        phaseRef.current += dt * 1.0;
        targets = Array.from({ length: BAR_COUNT }, (_, i) =>
          processingEnergy(i, BAR_COUNT, phaseRef.current),
        );
      }

      smoothedLevelsRef.current = tickLevels(
        smoothedLevelsRef.current,
        targets,
      );
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
      <div
        className={`recording-overlay ${isVisible ? "fade-in" : ""}`}
      >
        <div className="overlay-middle">
          {(state === "recording" || state === "transcribing") && (
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
          {state === "processing" && (
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
