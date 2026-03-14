interface UseHoverTimeoutOptions {
  timeoutMs?: number;
  onMouseEnter?: () => void;
  onMouseLeave?: () => void;
  selectors?: {
    item?: string | null;
    preview?: string | null;
  };
}

interface UseHoverTimeoutReturn {
  handleMouseEnter: () => void;
  handleMouseLeave: (e: React.MouseEvent) => void;
}

export function useHoverTimeout(
  clearHoverTimeout: () => void,
  setHoverTimeout: (timeout: number | null) => void,
  setHoveredItem: (item: null) => void,
  options: UseHoverTimeoutOptions = {}
): UseHoverTimeoutReturn {
  const {
    timeoutMs = 200,
    onMouseEnter,
    onMouseLeave,
    selectors = { item: '.flex.gap-4.p-3', preview: '.fixed.inset-x-0.bottom-0' }
  } = options;

  const handleMouseEnter = () => {
    clearHoverTimeout();
    onMouseEnter?.();
  };

  const handleMouseLeave = (e: React.MouseEvent) => {
    const relatedTarget = e.relatedTarget as HTMLElement | null;
    
    // Check if relatedTarget is actually an HTMLElement - it can be string "null" in some browsers/frameworks
    const isValidTarget = relatedTarget instanceof HTMLElement;
    
    const isMovingToItem = selectors.item && isValidTarget
      ? relatedTarget.closest(selectors.item) !== null 
      : false;
    const isMovingToPreview = selectors.preview && isValidTarget
      ? relatedTarget.closest(selectors.preview) !== null 
      : false;

    if (isMovingToItem || isMovingToPreview) {
      return;
    }

    onMouseLeave?.();

    const timeout = window.setTimeout(() => {
      setHoveredItem(null);
      setHoverTimeout(null);
    }, timeoutMs);
    setHoverTimeout(timeout);
  };

  return { handleMouseEnter, handleMouseLeave };
}
