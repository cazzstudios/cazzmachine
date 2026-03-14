import type { CrawlItem } from "../lib/tauri";
import { toggleSaveItem, openUrl } from "../lib/tauri";
import { useState } from "react";
import { useAppStore } from "../stores/appStore";
import { useHoverTimeout } from "../hooks/useHoverTimeout";
import { useThumbnailUrl } from "../hooks/useThumbnailUrl";

function timeAgo(dateStr: string): string {
  const now = new Date();
  const then = new Date(dateStr.replace(" ", "T"));
  const diffMs = now.getTime() - then.getTime();
  const minutes = Math.floor(diffMs / 60000);
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  return `${hours}h ago`;
}

interface Props {
  item: CrawlItem;
}

export function CrawlItemCard({ item }: Props) {
  const [saved, setSaved] = useState(item.is_saved);
  const imageUrl = useThumbnailUrl(item.thumbnail_data, item.thumbnail_url, { onError: 'silent' });
  const isDoneWorking = useAppStore((s) => s.isDoneWorking);
  const hoveredItem = useAppStore((s) => s.hoveredItem);
  const setHoveredItem = useAppStore((s) => s.setHoveredItem);
  const setHoverTimeout = useAppStore((s) => s.setHoverTimeout);
  const clearHoverTimeout = useAppStore((s) => s.clearHoverTimeout);
  const isHovered = hoveredItem?.id === item.id;

  const handleOpen = () => {
    if (!isDoneWorking) return;
    openUrl(item.url);
  };

  const handleSave = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      const newState = await toggleSaveItem(item.id);
      setSaved(newState);
    } catch {
      // noop
    }
  };

  const { handleMouseEnter, handleMouseLeave } = useHoverTimeout(
    clearHoverTimeout,
    setHoverTimeout,
    setHoveredItem,
    {
      timeoutMs: 200,
      onMouseEnter: () => {
        if (isDoneWorking) {
          setHoveredItem(item);
        }
      },
      selectors: {
        item: '.flex.gap-4.p-3',
        preview: '.preview-content'
      }
    }
  );

  return (
    <div
      onClick={handleOpen}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      className={`
        bg-cazz-surface border font-mono transition-all relative overflow-hidden
        ${isDoneWorking
          ? "cursor-pointer hover:border-cazz-accent/50 group"
          : "cursor-default opacity-60"}
        ${isHovered ? "border-cazz-accent shadow-sm" : "border-cazz-border"}
      `}
    >
      <div className="flex gap-4 p-3 relative z-10">
        {imageUrl && (
          <div className="flex-shrink-0 w-16 h-16 border border-cazz-border bg-black grayscale group-hover:grayscale-0 transition-all">
            <img
              src={imageUrl}
              alt=""
              className="w-full h-full object-cover opacity-80 group-hover:opacity-100"
              loading="lazy"
              onError={(e) => {
                const img = e.target as HTMLImageElement;
                img.style.display = "none";
                img.parentElement!.classList.add("flex", "items-center", "justify-center");
                img.parentElement!.innerHTML = "<span class='text-xs text-cazz-muted'>[IMG]</span>";
              }}
            />
          </div>
        )}
        <div className="flex-1 min-w-0 flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-2 mb-1.5">
              <span className="text-[10px] text-cazz-accent font-bold uppercase tracking-tighter bg-cazz-accent/10 px-1 border border-cazz-accent/20">
                {item.category}
              </span>
              <span className="text-[9px] text-cazz-muted uppercase tracking-widest truncate">
                SRC: {item.source}
              </span>
              <span className="text-[9px] text-cazz-muted ml-auto">
                [{timeAgo(item.fetched_at)}]
              </span>
            </div>
            <p className={`text-[13px] leading-tight line-clamp-2 uppercase font-medium ${
              isDoneWorking ? "text-cazz-text group-hover:text-cazz-highlight" : "text-cazz-muted"
            }`}>
              {item.title}
            </p>
          </div>
        </div>
        <button
          onClick={handleSave}
          className={`flex-shrink-0 text-sm self-start pt-1 transition-all ${
            saved ? "text-cazz-accent" : "text-cazz-muted/30 group-hover:text-cazz-muted"
          }`}
          title={saved ? "Unsave" : "Save for later"}
        >
          {saved ? "[★]" : "[ ]"}
        </button>
      </div>
      {isHovered && (
        <div className="absolute top-0 right-0 w-1 h-full bg-cazz-accent animate-pulse" />
      )}
    </div>
  );
}
