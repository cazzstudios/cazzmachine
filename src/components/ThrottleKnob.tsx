import { useState, useRef, useEffect } from "react";
import { useAppStore } from "../stores/appStore";

export function ThrottleKnob() {
  const throttleLevel = useAppStore((s) => s.throttleLevel);
  const setThrottleLevel = useAppStore((s) => s.setThrottleLevel);
  const [isDragging, setIsDragging] = useState(false);
  const knobRef = useRef<HTMLDivElement>(null);

  const updateLevelFromEvent = (clientX: number, clientY: number) => {
    if (!knobRef.current) return;

    const rect = knobRef.current.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    const angleRad = Math.atan2(clientY - centerY, clientX - centerX);
    const rawAngleDeg = (angleRad * 180) / Math.PI;

    let normalizedDeg = rawAngleDeg + 90;
    if (normalizedDeg > 180) normalizedDeg -= 360;
    if (normalizedDeg < -180) normalizedDeg += 360;

    const clampedDeg = Math.max(-135, Math.min(135, normalizedDeg));

    const newLevel = Math.round(((clampedDeg + 135) / 270) * 9) + 1;
    if (newLevel !== throttleLevel) {
      setThrottleLevel(newLevel);
    }
  };

  const handleStart = (e: React.MouseEvent | React.TouchEvent) => {
    e.preventDefault();
    setIsDragging(true);

    if ("touches" in e) {
      const touch = e.touches[0];
      updateLevelFromEvent(touch.clientX, touch.clientY);
    } else {
      const mouseEvent = e as React.MouseEvent;
      updateLevelFromEvent(mouseEvent.clientX, mouseEvent.clientY);
    }
  };

  useEffect(() => {
    const handleMove = (e: MouseEvent | TouchEvent) => {
      if (!isDragging) return;

      if ("touches" in e) {
        const touch = e.touches[0];
        updateLevelFromEvent(touch.clientX, touch.clientY);
      } else {
        updateLevelFromEvent(e.clientX, e.clientY);
      }
    };

    const handleEnd = () => {
      setIsDragging(false);
    };

    if (isDragging) {
      document.addEventListener("mousemove", handleMove);
      document.addEventListener("touchmove", handleMove, { passive: false });
      document.addEventListener("mouseup", handleEnd);
      document.addEventListener("touchend", handleEnd);
    }

    return () => {
      document.removeEventListener("mousemove", handleMove);
      document.removeEventListener("touchmove", handleMove);
      document.removeEventListener("mouseup", handleEnd);
      document.removeEventListener("touchend", handleEnd);
    };
  }, [isDragging, throttleLevel]);

  const rotation = ((throttleLevel - 1) / 8) * 270 - 135;

  return (
    <div className="flex flex-col items-center">
      <div className="relative w-20 h-20">
        <div
          ref={knobRef}
          className={`w-20 h-20 rounded-full bg-cazz-surface border-2 cursor-pointer transition-all ${
            isDragging
              ? "border-cazz-accent scale-105 shadow-lg shadow-cazz-accent/20"
              : "border-cazz-border hover:border-cazz-muted/50"
          }`}
          style={{ transform: `rotate(${rotation}deg)` }}
          onMouseDown={handleStart}
          onTouchStart={handleStart}
        >
          <div className="absolute top-2 left-1/2 -translate-x-1/2 w-1.5 h-3 bg-cazz-accent rounded-full pointer-events-none" />
        </div>
        
        <span className="absolute -top-6 left-1/2 -translate-x-1/2 text-[10px] font-mono text-cazz-muted uppercase tracking-wider whitespace-nowrap">
          mid cazz
        </span>

        <span className="absolute top-[68px] -left-12 text-[10px] font-mono text-cazz-muted uppercase tracking-wider whitespace-nowrap">
          low cazz
        </span>

        <span className="absolute top-[68px] -right-12 text-[10px] font-mono text-cazz-muted uppercase tracking-wider whitespace-nowrap">
          top cazz
        </span>
      </div>
    </div>
  );
}
