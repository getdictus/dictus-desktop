import React from "react";
import i18n from "@/i18n";

interface ErrorBoundaryProps {
  children: React.ReactNode;
}

interface ErrorBoundaryState {
  error: Error | null;
}

/**
 * Catches uncaught render errors so a single component failure shows a
 * recoverable fallback instead of unmounting the whole React tree (which
 * appears as a blank/black window in the Tauri webview).
 */
export class ErrorBoundary extends React.Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    // Surface to the dev console / terminal so the real trigger is visible.
    console.error("Uncaught UI error:", error, info.componentStack);
  }

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (this.state.error) {
      return (
        <div className="flex flex-col items-center justify-center h-screen gap-4 p-8 text-center">
          <p className="text-lg font-medium text-text">
            {i18n.t("errors.boundary.title")}
          </p>
          <p className="text-sm text-text/60 max-w-md break-words">
            {this.state.error.message}
          </p>
          <button
            type="button"
            onClick={this.handleReload}
            className="px-4 py-2 text-sm font-medium rounded-md bg-logo-primary text-white hover:bg-logo-primary/90 focus:outline-none"
          >
            {i18n.t("errors.boundary.reload")}
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
