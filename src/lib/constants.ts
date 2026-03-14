// CazzMachine Constants
// Extracted magic numbers for maintainability

// Throttle levels
export const MIN_THROTTLE_LEVEL = 1;
export const MAX_THROTTLE_LEVEL = 9;

// Thread counts
export const MIN_THREAD_COUNT = 1;
export const MAX_THREAD_COUNT = 8;

// Durations (ms)
export const TOAST_DURATION_MS = 5000;
export const INTERRUPTED_STATE_DELAY_MS = 2000;
export const HOVER_TIMEOUT_MS = 200;
export const STATS_REFRESH_INTERVAL_MS = 30000;

// Content time costs (minutes per item)
export const CONTENT_COSTS = {
  meme: 0.5,
  joke: 0.3,
  news: 2.0,
  video: 3.0,
  gossip: 1.5,
} as const;

// Doomscroll formula constants
export const DOOMSCROLL_CONFIG = {
  MIN_MINUTES: 1,
  MAX_MINUTES: 5,
  MULTIPLIER: 4,
  DIVISOR: 8,
} as const;
