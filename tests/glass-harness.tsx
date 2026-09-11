import React from "react";
import ReactDOM from "react-dom/client";
import {
  GlassLens,
  glassSupport,
  opticsFor,
  SWITCH_BASE,
  SWITCH_LIGHT,
  SWITCH_DARK,
} from "../src/components/ui/glass";
import "../src/App.css";

/**
 * Mounts one lens under React.StrictMode and lets the test unmount and remount
 * it, which is the cycle that kills the WebGL context upstream
 * (samasante/liquid-glass#2). Everything the test asserts on is exposed through
 * data attributes rather than pixels, because a headless run does not composite
 * the filter.
 */
const Harness: React.FC = () => {
  const [mounted, setMounted] = React.useState(true);

  return (
    <div style={{ padding: 40, background: "#f5f6f9" }}>
      <button id="toggle" onClick={() => setMounted((value) => !value)}>
        toggle
      </button>
      <span id="support" data-support={glassSupport()}>
        {glassSupport()}
      </span>
      <div id="host" style={{ marginTop: 20 }}>
        {mounted && (
          <GlassLens
            data-testid="lens"
            behind="#f5f6f9"
            width={20}
            height={20}
            radius={10}
            optics={opticsFor(SWITCH_BASE, SWITCH_LIGHT, SWITCH_DARK, false)}
          />
        )}
      </div>
    </div>
  );
};

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Harness />
  </React.StrictMode>,
);
