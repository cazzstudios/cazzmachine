import { useAppStore } from "../stores/appStore";
import type { Category } from "../lib/tauri";

const categories: Array<{ key: Category | "all"; label: string; icon: string }> = [
  { key: "all", label: "ALL_SYSTEMS", icon: "ALL" },
  { key: "meme", label: "MEME_LOG", icon: "MEM" },
  { key: "joke", label: "JOKE_BUF", icon: "JOK" },
  { key: "news", label: "NEWS_FEED", icon: "NWS" },
  { key: "video", label: "VID_STRM", icon: "VID" },
  { key: "gossip", label: "GOSSIP_DMP", icon: "GOS" },
];

export function CategoryFilter() {
  const active = useAppStore((s) => s.activeCategory);
  const setActive = useAppStore((s) => s.setActiveCategory);

  return (
    <div className="flex gap-2 overflow-x-auto pb-4 px-1 scrollbar-hide">
      {categories.map((cat) => (
        <button
          key={cat.key}
          onClick={() => setActive(cat.key)}
          className={`flex items-center gap-2 px-3 py-1.5 border font-mono text-[10px] uppercase tracking-wider transition-all ${
            active === cat.key
              ? "bg-cazz-accent text-white border-cazz-accent shadow-sm"
              : "bg-cazz-surface text-cazz-muted border-cazz-border hover:border-cazz-muted/50"
          }`}
        >
          <span className="font-bold opacity-60">[{cat.icon}]</span>
          <span>{cat.label}</span>
        </button>
      ))}
    </div>
  );
}
