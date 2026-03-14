import { useAppStore } from "../stores/appStore";
import { CategoryFilter } from "./CategoryFilter";
import { CrawlItemCard } from "./CrawlItemCard";

export function DetailView() {
  const items = useAppStore((s) => s.items);
  const isLoading = useAppStore((s) => s.isLoading);
  const setView = useAppStore((s) => s.setView);
  const setIsDoneWorking = useAppStore((s) => s.setIsDoneWorking);

  return (
    <div className="animate-fade-in space-y-4 pb-20">
      <div className="flex items-center justify-between border-b border-cazz-border pb-2">
        <div>
          <h2 className="text-xl font-bold tracking-tighter text-cazz-text uppercase italic">
            DATA_DUMP.LOG
          </h2>
          <p className="text-[9px] font-mono text-cazz-muted uppercase tracking-widest">
            Session: {new Date().toISOString().split('T')[0]} // Status: RECOVERED
          </p>
        </div>
        <button
          onClick={() => {
            setIsDoneWorking(false);
            setView("idle");
          }}
          className="px-2 py-1 border border-cazz-border text-[8px] md:text-[10px] font-mono text-cazz-muted hover:text-cazz-accent hover:border-cazz-accent transition-all uppercase whitespace-nowrap"
        >
          [ ESC_TERMINAL ]
        </button>
      </div>

      <CategoryFilter />

      {isLoading ? (
        <div className="text-center py-12 font-mono">
          <div className="text-cazz-accent animate-pulse mb-2 tracking-[0.3em] uppercase text-xs font-bold">
            Synchronizing...
          </div>
          <div className="w-48 h-1 bg-cazz-surface border border-cazz-border mx-auto overflow-hidden">
            <div className="h-full bg-cazz-accent animate-progress-indefinite w-1/2" />
          </div>
        </div>
      ) : items.length === 0 ? (
        <div className="text-center py-16 border-2 border-dashed border-cazz-border rounded-xl">
          <div className="text-cazz-muted font-mono uppercase text-xs tracking-widest">
            ERROR: Buffer Empty
          </div>
          <p className="text-[10px] font-mono text-cazz-muted/60 mt-2">
            Engines operating at low cazz. Try increasing the throttle.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {items.map((item) => (
            <CrawlItemCard key={item.id} item={item} />
          ))}
          <div className="border-t border-dashed border-cazz-border pt-4 text-center">
            <p className="text-[10px] font-mono text-cazz-muted uppercase tracking-[0.2em]">
              EOF // {items.length} Entries Recovered // END_OF_LOG
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
