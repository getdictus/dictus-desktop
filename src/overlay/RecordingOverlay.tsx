import { listen } from "@tauri-apps/api/event";
import React, { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  MicrophoneIcon,
  TranscriptionIcon,
  CancelIcon,
} from "../components/icons";
import "./RecordingOverlay.css";
import { commands } from "@/bindings";
import i18n, { syncLanguageFromSettings } from "@/i18n";
import { getLanguageDirection } from "@/lib/utils/rtl";

type OverlayState = "recording" | "transcribing" | "processing";

const BAR_COUNT = 18;

function interpolateLevels(source: number[], barCount: number): number[] {
  if (source.length === 0) return Array(barCount).fill(0);
  return Array.from({ length: barCount }, (_, index) => {
    const position = index / Math.max(barCount - 1, 1);
    const arrayIndex = position * (source.length - 1);
    const lower = Math.floor(arrayIndex);
    const upper = Math.min(lower + 1, source.length - 1);
    const fraction = arrayIndex - lower;
    const value = source[lower] * (1 - fraction) + source[upper] * fraction;
    return value < 0.05 ? 0 : Math.min(Math.max(value, 0), 1);
  });
}

function tickLevels(current: number[], targets: number[]): number[] {
  return current.map((prev, i) => {
    const target = targets[i] ?? 0;
    let next: number;
    if (target > prev) {
      next = prev + (target - prev) * 0.3; // rise: smooth
    } else {
      next = target + (prev - target) * 0.85; // fall: decay
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
    return "#6BA3FF"; // blue gradient — inner 40%
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
      // Listen for show-overlay event from Rust
      const unlistenShow = await listen("show-overlay", async (event) => {
        // Sync language from settings each time overlay is shown
        await syncLanguageFromSettings();
        const overlayState = event.payload as OverlayState;
        setState(overlayState);
        // Reset animation state on show
        phaseRef.current = 0;
        smoothedLevelsRef.current = Array(BAR_COUNT).fill(0);
        targetLevelsRef.current = Array(BAR_COUNT).fill(0);
        setIsVisible(true);
      });

      // Listen for hide-overlay event from Rust
      const unlistenHide = await listen("hide-overlay", () => {
        cancelAnimationFrame(rafIdRef.current);
        setIsVisible(false);
      });

      // Listen for mic-level updates — store interpolated targets
      const unlistenLevel = await listen<number[]>("mic-level", (event) => {
        const raw = event.payload as number[];
        targetLevelsRef.current = interpolateLevels(raw, BAR_COUNT);
      });

      // Cleanup function
      return () => {
        unlistenShow();
        unlistenHide();
        unlistenLevel();
      };
    };

    setupEventListeners();
  }, []);

  // Animation loop — runs when visible
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
        // transcribing or processing: sine wave animation
        phaseRef.current += dt * 0.5;
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

  const getIcon = () => {
    if (state === "recording") {
      return <MicrophoneIcon color="#EF4444" />;
    } else {
      return <TranscriptionIcon />;
    }
  };

  return (
    <div
      dir={direction}
      className={`recording-overlay ${isVisible ? "fade-in" : ""}`}
    >
      <div className="overlay-left">{getIcon()}</div>

      <div className="overlay-middle">
        {(state === "recording" || state === "transcribing") && (
          <div className="bars-container">
            {levels.map((v, i) => (
              <div
                key={i}
                className="bar"
                style={{
                  height: `${Math.max(3, v * 36)}px`,
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

      <div className="overlay-right">
        {state === "recording" && (
          <div
            className="cancel-button"
            onClick={() => {
              commands.cancelOperation();
            }}
          >
            <CancelIcon />
          </div>
        )}
      </div>
    </div>
  );
};

export default RecordingOverlay;
