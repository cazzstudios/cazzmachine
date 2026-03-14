import { useMemo } from 'react';

interface ThreadSliderProps {
  value: number;
  onChange: (value: number) => void;
  disabled?: boolean;
}

function debounce<T extends (...args: any[]) => void>(fn: T, delay: number) {
  let timeoutId: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

export function getHeatColor(level: number): string {
  const colors = [
    '#3b82f6', '#06b6d4', '#14b8a6', '#22c55e', '#eab308', '#f97316', '#fb7185', '#dc2626'
  ];
  return colors[level - 1] || colors[0];
}

export function ThreadSlider({ value, onChange, disabled }: ThreadSliderProps) {
  const debouncedOnChange = useMemo(
    () => debounce((newValue: number) => onChange(newValue), 300),
    [onChange]
  );

  const handleIncrement = () => {
    if (!disabled && value < 8) {
      debouncedOnChange(value + 1);
    }
  };

  const handleDecrement = () => {
    if (!disabled && value > 1) {
      debouncedOnChange(value - 1);
    }
  };

  const getLabel = () => {
    if (value === 1) return { text: 'single_cazz', color: 'rgb(107, 114, 128)' };
    if (value === 2) return { text: 'double_cazz', color: getHeatColor(value) };
    if (value >= 5 && value <= 7) return { text: 'high_cazz', color: getHeatColor(value) };
    if (value === 8) return { text: 'full_cazz', color: getHeatColor(8) };
    return { text: 'multi_cazz', color: getHeatColor(value) };
  };

  const label = getLabel();

  return (
    <div className="flex flex-col items-center">
      <div className="flex items-center gap-2">
        {/* Minus button */}
        <button
          onClick={handleDecrement}
          disabled={disabled || value <= 1}
          className="w-8 h-8 rounded-lg bg-cazz-card border border-cazz-border/50 
            text-cazz-muted hover:text-cazz-accent hover:border-cazz-accent/50
            disabled:opacity-30 disabled:cursor-not-allowed transition-all duration-200
            flex items-center justify-center font-mono text-lg"
        >
          −
        </button>

        {/* Heat bars column */}
        <div className="flex flex-col-reverse gap-1">
          {Array.from({ length: 8 }).map((_, i) => {
            const level = i + 1;
            const isActive = level <= value;
            const color = getHeatColor(level);
            
            return (
              <button
                key={level}
                onClick={() => !disabled && debouncedOnChange(level)}
                disabled={disabled}
                className={`
                  w-16 h-3 rounded-sm transition-all duration-200
                  disabled:opacity-30 disabled:cursor-not-allowed
                `}
                style={{
                  backgroundColor: isActive ? color : '#1e293b',
                  boxShadow: isActive ? `0 0 8px ${color}60` : 'none',
                  opacity: isActive ? 1 : 0.4,
                }}
              />
            );
          })}
        </div>

        {/* Plus button */}
        <button
          onClick={handleIncrement}
          disabled={disabled || value >= 8}
          className="w-8 h-8 rounded-lg bg-cazz-card border border-cazz-border/50 
            text-cazz-muted hover:text-cazz-accent hover:border-cazz-accent/50
            disabled:opacity-30 disabled:cursor-not-allowed transition-all duration-200
            flex items-center justify-center font-mono text-lg"
        >
          +
        </button>
      </div>

      <span 
        className="text-[10px] font-mono uppercase tracking-wider mt-2 transition-colors duration-300"
        style={{ color: label.color }}
      >
        {label.text}
      </span>
    </div>
  );
}
