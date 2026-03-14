import { useAppStore } from "../stores/appStore";
import { openUrl } from "../lib/tauri";
import { useThumbnailUrl } from "../hooks/useThumbnailUrl";
import { useHoverTimeout } from "../hooks/useHoverTimeout";

export function HoverPreview() {
  const hoveredItem = useAppStore((s) => s.hoveredItem);
  const isDoneWorking = useAppStore((s) => s.isDoneWorking);
  const setHoveredItem = useAppStore((s) => s.setHoveredItem);
  const clearHoverTimeout = useAppStore((s) => s.clearHoverTimeout);
  const setHoverTimeout = useAppStore((s) => s.setHoverTimeout);
  const imageUrl = useThumbnailUrl(
    hoveredItem?.thumbnail_data,
    hoveredItem?.thumbnail_url,
    { onError: 'null' }
  );

  if (!hoveredItem || !isDoneWorking) return null;

  const { handleMouseEnter, handleMouseLeave } = useHoverTimeout(
    clearHoverTimeout,
    setHoverTimeout,
    setHoveredItem,
    {
      timeoutMs: 200,
      selectors: {
        item: '.flex.gap-4.p-3',
        preview: '.preview-content'
      }
    }
  );

  const handleOpen = () => {
    openUrl(hoveredItem.url);
  };

  const jokeText = hoveredItem.description || hoveredItem.title;
  const showTitle = hoveredItem.category !== "joke";

  return (
    <div
      className="fixed inset-x-0 bottom-0 z-40 animate-slide-up px-4 pb-4 cursor-pointer"
      onClick={handleOpen}
    >
      <div 
        className="preview-content bg-cazz-surface border-2 border-dashed border-cazz-accent shadow-2xl max-w-2xl mx-auto overflow-hidden"
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
      >
        <div className="bg-cazz-accent/10 border-b border-dashed border-cazz-accent/30 px-3 py-1 flex justify-between items-center">
          <span className="text-[10px] font-mono font-bold text-cazz-accent uppercase tracking-widest">
            [PREVIEW_TERMINAL // CAT:{hoveredItem.category.toUpperCase()}]
          </span>
          <span className="text-[9px] font-mono text-cazz-accent/60">
            ID:{hoveredItem.id.slice(0, 8)}
          </span>
        </div>
        
        <div className="p-5">
          {/* Jokes: large centered text */}
          {hoveredItem.category === "joke" && (
            <div className="text-center font-mono">
              <p className="text-lg text-cazz-text italic leading-relaxed">"{jokeText}"</p>
              <div className="mt-4 pt-4 border-t border-dashed border-cazz-border flex justify-center gap-4 text-[9px] text-cazz-muted uppercase tracking-[0.2em]">
                <span>SRC: {hoveredItem.source}</span>
                <span>[ CLICK TO EXECUTE_OPEN ]</span>
              </div>
            </div>
          )}

          {/* Memes: large image */}
          {hoveredItem.category === "meme" && (
            <div className="text-center">
              {imageUrl && (
                <div className="inline-block border border-cazz-border p-1 bg-black mb-3">
                  <img
                    src={imageUrl}
                    alt={hoveredItem.title}
                    className="max-h-64 w-auto grayscale-0"
                    onError={(e) => {
                      (e.target as HTMLImageElement).style.display = "none";
                    }}
                  />
                </div>
              )}
              {showTitle && hoveredItem.title && (
                <p className="text-cazz-text font-mono text-sm uppercase tracking-tight">{hoveredItem.title}</p>
              )}
              <div className="mt-4 pt-4 border-t border-dashed border-cazz-border flex justify-center gap-4 text-[9px] text-cazz-muted uppercase tracking-[0.2em]">
                <span>SRC: {hoveredItem.source}</span>
                <span>[ CLICK TO EXECUTE_OPEN ]</span>
              </div>
            </div>
          )}

          {/* News: image + text */}
          {hoveredItem.category === "news" && (
            <div className="flex gap-6">
              {imageUrl && (
                <div className="flex-shrink-0 border border-cazz-border p-1 bg-black h-fit">
                  <img
                    src={imageUrl}
                    alt={hoveredItem.title}
                    className="w-40 h-32 object-cover"
                    onError={(e) => {
                      (e.target as HTMLImageElement).style.display = "none";
                    }}
                  />
                </div>
              )}
              <div className="flex-1 min-w-0 font-mono">
                {showTitle && hoveredItem.title && (
                  <h3 className="font-bold text-cazz-text text-base mb-3 uppercase tracking-tighter leading-none">{hoveredItem.title}</h3>
                )}
                {hoveredItem.description && (
                  <p className="text-cazz-muted text-[13px] leading-snug line-clamp-4">{hoveredItem.description}</p>
                )}
                <div className="mt-4 pt-4 border-t border-dashed border-cazz-border flex gap-4 text-[9px] text-cazz-muted uppercase tracking-[0.2em]">
                  <span>SRC: {hoveredItem.source}</span>
                  <span>[ CLICK TO EXECUTE_OPEN ]</span>
                </div>
              </div>
            </div>
          )}

          {/* Videos: thumbnail + title */}
          {hoveredItem.category === "video" && (
            <div className="flex gap-6">
              {imageUrl && (
                <div className="flex-shrink-0 border border-cazz-border p-1 bg-black h-fit">
                  <img
                    src={imageUrl}
                    alt={hoveredItem.title}
                    className="w-40 h-24 object-cover"
                    onError={(e) => {
                      (e.target as HTMLImageElement).style.display = "none";
                    }}
                  />
                </div>
              )}
              <div className="flex-1 min-w-0 font-mono">
                {showTitle && hoveredItem.title && (
                  <p className="text-cazz-text font-bold text-base uppercase tracking-tighter leading-tight">{hoveredItem.title}</p>
                )}
                <div className="mt-4 pt-4 border-t border-dashed border-cazz-border flex gap-4 text-[9px] text-cazz-muted uppercase tracking-[0.2em]">
                  <span>SRC: {hoveredItem.source}</span>
                  <span>[ CLICK TO EXECUTE_OPEN ]</span>
                </div>
              </div>
            </div>
          )}

          {/* Gossip: title */}
          {hoveredItem.category === "gossip" && (
            <div className="text-center font-mono">
              <p className="text-cazz-text text-lg font-bold uppercase tracking-tighter italic">
                {showTitle ? hoveredItem.title : "SENSATIONAL_INTEL"}
              </p>
              <div className="mt-4 pt-4 border-t border-dashed border-cazz-border flex justify-center gap-4 text-[9px] text-cazz-muted uppercase tracking-[0.2em]">
                <span>SRC: {hoveredItem.source}</span>
                <span>[ CLICK TO EXECUTE_OPEN ]</span>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
