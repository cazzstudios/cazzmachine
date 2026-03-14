import { useAppStore } from "../stores/appStore";

export function Toast() {
  const message = useAppStore((s) => s.toastMessage);
  const dismiss = useAppStore((s) => s.setToastMessage);

  if (!message) return null;

  return (
    <div className="fixed top-4 right-4 z-50 animate-slide-in max-w-sm">
      <div className="bg-cazz-surface border-2 border-cazz-accent p-3 shadow-xl font-mono">
        <div className="flex items-start gap-4">
          <div className="flex-1">
            <div className="text-[10px] text-cazz-accent font-bold uppercase tracking-widest mb-1">
              [SYSTEM_NOTIFICATION]
            </div>
            <p className="text-xs text-cazz-text leading-snug">{message}</p>
          </div>
          <button
            onClick={() => dismiss(null)}
            className="text-[10px] text-cazz-muted hover:text-cazz-text transition-colors border border-cazz-border px-1 uppercase"
          >
            [X]
          </button>
        </div>
      </div>
    </div>
  );
}
