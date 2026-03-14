import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("[ErrorBoundary] Uncaught error:", error);
    console.error("[ErrorBoundary] Component stack:", errorInfo.componentStack);
  }

  private handleReload = () => {
    window.location.reload();
  };

  public render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen bg-cazz-bg text-cazz-text flex items-center justify-center p-6">
          <div className="max-w-md w-full text-center">
            <div className="mb-6">
              <svg
                className="w-16 h-16 mx-auto text-red-500"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                />
              </svg>
            </div>
            <h1 className="text-2xl font-bold mb-4 font-mono">
              Something went wrong
            </h1>
            <p className="text-cazz-muted mb-6">
              The Cazzmachine encountered an unexpected error. Don't worry, your data is safe.
            </p>
            <button
              onClick={this.handleReload}
              className="px-6 py-3 bg-cazz-accent text-cazz-bg font-mono font-bold rounded hover:opacity-90 transition-opacity"
            >
              Reload App
            </button>
            {import.meta.env.DEV && this.state.error && (
              <div className="mt-6 p-4 bg-red-900/20 rounded text-left overflow-auto">
                <p className="text-red-400 font-mono text-sm mb-2">Error details (dev mode only):</p>
                <pre className="text-red-300 font-mono text-xs whitespace-pre-wrap">
                  {this.state.error.toString()}
                </pre>
              </div>
            )}
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
