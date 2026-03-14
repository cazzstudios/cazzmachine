import { useState, useEffect, useCallback } from "react";
import { useAppStore } from "../stores/appStore";
import {
  getDiagnosticSummary,
  getProviderStatus,
  getRecentDiagnostics,
  type DiagnosticSummary,
  type ProviderStatus,
  type DiagnosticLog,
} from "../lib/tauri";

interface DiagnosticsPanelProps {
  onClose: () => void;
}

export function DiagnosticsPanel({ onClose }: DiagnosticsPanelProps) {
  const setToastMessage = useAppStore((s) => s.setToastMessage);

  const providerLabels: Record<string, string> = {
    dadjokes: "Reddit: r/dadjokes",
    entertainment: "Reddit: r/entertainment",
    "icanhazdadjoke": "icanhazdadjoke.com",
    "google-news": "Google News",
    memes: "Reddit: r/memes",
    "reddit-memes": "Reddit: r/dankmemes",
    "reddit-videos": "Reddit: r/videos",
    gossip: "Reddit: r/popculturechat",
  };

  const getProviderLabel = (name: string) => providerLabels[name] || name;
  const [summary, setSummary] = useState<DiagnosticSummary | null>(null);
  const [providers, setProviders] = useState<ProviderStatus[]>([]);
  const [logs, setLogs] = useState<DiagnosticLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [sum, prov, diag] = await Promise.all([
        getDiagnosticSummary(),
        getProviderStatus(),
        getRecentDiagnostics(50),
      ]);
      setSummary(sum);
      setProviders(prov);
      setLogs(diag);
    } catch (e) {
      setError(`Failed to load diagnostics: ${e}`);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleExport = async () => {
    const data = {
      timestamp: new Date().toISOString(),
      summary,
      providers,
      logs,
    };
    const json = JSON.stringify(data, null, 2);
    await navigator.clipboard.writeText(json);
    setToastMessage("Diagnostics copied to clipboard!");
  };

  if (loading) {
    return (
      <div className="fixed inset-0 bg-cazz-bg/95 z-50 flex items-center justify-center">
        <div className="text-cazz-text font-mono">Loading diagnostics...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="fixed inset-0 bg-cazz-bg/95 z-50 flex items-center justify-center">
        <div className="text-red-400 font-mono max-w-md text-center">
          <p className="mb-4">{error}</p>
          <button
            onClick={onClose}
            className="px-4 py-2 bg-cazz-accent text-cazz-bg rounded"
          >
            Close
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-cazz-bg/95 z-50 overflow-auto">
      <div className="max-w-4xl mx-auto p-6">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-2xl font-bold font-mono text-cazz-text">
            Diagnostics Panel
          </h1>
          <div className="flex gap-2">
            <button
              onClick={loadData}
              className="px-4 py-2 bg-cazz-accent/20 text-cazz-accent rounded font-mono hover:bg-cazz-accent/30"
            >
              Refresh
            </button>
            <button
              onClick={handleExport}
              className="px-4 py-2 bg-cazz-accent text-cazz-bg rounded font-mono hover:opacity-90"
            >
              Export JSON
            </button>
            <button
              onClick={onClose}
              className="px-4 py-2 bg-red-500/20 text-red-400 rounded font-mono hover:bg-red-500/30"
            >
              Close
            </button>
          </div>
        </div>

        {/* System Status */}
        <section className="mb-6 p-4 border border-cazz-border rounded">
          <h2 className="text-lg font-bold font-mono mb-4 text-cazz-accent">
            System Status
          </h2>
          {summary && (
            <div className="grid grid-cols-2 gap-y-2 font-mono text-sm">
              <div>
                <span className="text-cazz-muted">Buffer Health:</span>{" "}
                <span
                  className={`${
                    summary.estimated_buffer_health === "healthy"
                      ? "text-green-400"
                      : summary.estimated_buffer_health === "low"
                        ? "text-yellow-400"
                        : "text-red-400"
                  }`}
                >
                  {summary.estimated_buffer_health} ({summary.pending_count} items)
                </span>
              </div>
              <div>
                <span className="text-cazz-muted">Buffer Time:</span>{" "}
                <span className="text-cazz-text">
                  {summary.budget_analysis.estimated_buffer_minutes.toFixed(2)} min
                </span>
              </div>
              <div>
                <span className="text-cazz-muted">Min Cost/Item:</span>{" "}
                <span className="text-cazz-text">
                  {summary.budget_analysis.min_cost_per_item.toFixed(2)} min
                </span>
              </div>
              <div>
                <span className="text-cazz-muted">Max Cost/Item:</span>{" "}
                <span className="text-cazz-text">
                  {summary.budget_analysis.max_cost_per_item.toFixed(2)} min
                </span>
              </div>
            </div>
          )}
        </section>

        {/* Provider Status */}
        <section className="mb-6 p-4 border border-cazz-border rounded">
          <h2 className="text-lg font-bold font-mono mb-4 text-cazz-accent">
            Provider Health
          </h2>
          <div className="overflow-x-auto">
            <table className="w-full font-mono text-sm">
              <thead>
                <tr className="text-cazz-muted border-b border-cazz-border">
                  <th className="text-left py-2 px-3">Provider</th>
                  <th className="text-left py-2 px-3">Category</th>
                  <th className="text-left py-2 px-3">Status</th>
                  <th className="text-left py-2 px-3">Last Fetch</th>
                  <th className="text-right py-2 px-3">Errors</th>
                  <th className="text-right py-2 px-3">Score</th>

                </tr>
              </thead>
              <tbody>
                {providers.map((provider) => (
                  <tr
                    key={`${provider.provider_name}-${provider.category}`}
                    className="border-b border-cazz-border/50"
                  >
                    <td className="py-2 px-3 text-cazz-text">
                      {getProviderLabel(provider.provider_name)}
                    </td>
                    <td className="py-2 px-3 text-cazz-muted">{provider.category}</td>
                    <td className="py-2 px-3">
                      <span
                        className={`${
                          provider.last_fetch_status === "success"
                            ? "text-green-400"
                            : provider.last_fetch_status === "error"
                              ? "text-red-400"
                              : "text-yellow-400"
                        }`}
                      >
                        {provider.last_fetch_status}
                      </span>
                    </td>
                    <td className="py-2 px-3 text-cazz-muted">
                      {provider.last_fetch_timestamp
                        ? new Date(provider.last_fetch_timestamp).toLocaleString()
                        : "Never"}
                    </td>
                    <td className="py-2 px-3 text-right">
                      {provider.recent_error_count > 0 ? (
                        <span className="text-red-400">
                          {provider.recent_error_count}
                        </span>
                      ) : (
                        <span className="text-green-400">0</span>
                      )}
                    </td>
                    <td className="py-2 px-3 text-right">
                      <span
                        className={`${
                          provider.score >= 0.7
                            ? "text-green-400"
                            : provider.score >= 0.4
                              ? "text-yellow-400"
                              : "text-red-400"
                        }`}
                      >
                        {provider.score.toFixed(2)}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Recent Diagnostic Events */}
        <section className="p-4 border border-cazz-border rounded">
          <h2 className="text-lg font-bold font-mono mb-4 text-cazz-accent">
            Recent Diagnostic Events
          </h2>
          <div className="overflow-x-auto max-h-[70vh] overflow-y-auto">
            <table className="w-full font-mono text-sm">
              <thead className="sticky top-0 bg-cazz-bg">
                <tr className="text-cazz-muted border-b border-cazz-border">
                  <th className="text-left py-2 px-3">Time</th>
                  <th className="text-left py-2 px-3">Type</th>
                  <th className="text-left py-2 px-3">Severity</th>
                  <th className="text-left py-2 px-3">Message</th>

                </tr>
              </thead>
              <tbody>
                {logs.length === 0 ? (
                  <tr>
                    <td
                      colSpan={4}
                      className="py-4 text-center text-cazz-muted"
                    >
                      No diagnostic events recorded
                    </td>
                  </tr>
                ) : (
                  logs.map((log) => (
                    <tr
                      key={log.id}
                      className="border-b border-cazz-border/50"
                    >
                      <td className="py-2 px-3 text-cazz-muted whitespace-nowrap">
                        {new Date(log.timestamp).toLocaleString()}
                      </td>
                      <td className="py-2 px-3 text-cazz-text">{log.event_type}</td>
                      <td className="py-2 px-3">
                        <span
                          className={`${
                            log.severity === "error"
                              ? "text-red-400"
                              : log.severity === "warn"
                                ? "text-yellow-400"
                                : "text-green-400"
                          }`}
                        >
                          {log.severity}
                        </span>
                      </td>
                      <td className="py-2 px-3 text-cazz-text max-w-md truncate">
                        {log.message}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </section>
      </div>
    </div>
  );
}
