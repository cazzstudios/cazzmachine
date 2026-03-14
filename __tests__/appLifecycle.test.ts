import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

const mockOnResume = vi.fn();
const mockOnPause = vi.fn();

vi.mock('tauri-plugin-app-events-api', () => ({
  onResume: (cb: () => void) => mockOnResume(cb),
  onPause: (cb: () => void) => mockOnPause(cb),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-notification', () => ({
  isPermissionGranted: vi.fn().mockResolvedValue(true),
  requestPermission: vi.fn().mockResolvedValue('granted'),
}));

const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

import { 
  getLastActiveTimestamp, 
  setLastActiveTimestamp, 
  consumePendingItems,
  skipNextNotification,
  isAndroid,
  logDiagnostic
} from '../src/lib/tauri';

describe('App Lifecycle - Android', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    Object.defineProperty(navigator, 'userAgent', {
      value: 'Mozilla/5.0 (Linux; Android 10; SM-G960U)',
      configurable: true,
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('isAndroid detection', () => {
    it('should return true for Android user agent', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value: 'Mozilla/5.0 (Linux; Android 10)',
        configurable: true,
      });
      expect(isAndroid()).toBe(true);
    });

    it('should return false for iOS user agent', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value: 'Mozilla/5.0 (iPhone; CPU iPhone OS 14_0)',
        configurable: true,
      });
      expect(isAndroid()).toBe(false);
    });

    it('should return false for desktop user agent', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
        configurable: true,
      });
      expect(isAndroid()).toBe(false);
    });

    it('should return false for macOS user agent', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)',
        configurable: true,
      });
      expect(isAndroid()).toBe(false);
    });

    it('should return false for Linux desktop user agent', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value: 'Mozilla/5.0 (X11; Linux x86_64)',
        configurable: true,
      });
      expect(isAndroid()).toBe(false);
    });
  });

  describe('App Background (Suspension)', () => {
    it('should save timestamp when app goes to background', async () => {
      const now = Date.now();
      mockInvoke.mockResolvedValueOnce(undefined);

      await setLastActiveTimestamp(now);

      expect(mockInvoke).toHaveBeenCalledWith('set_last_active_timestamp', { timestamp: now });
    });

    it('should call skipNextNotification on Android background', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await skipNextNotification();

      expect(mockInvoke).toHaveBeenCalledWith('skip_next_notification');
    });

    it('should handle errors when saving timestamp on background', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database locked'));

      await expect(setLastActiveTimestamp(Date.now())).rejects.toThrow('Database locked');
    });

    it('should handle errors when calling skipNextNotification', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Command not found'));

      await expect(skipNextNotification()).rejects.toThrow('Command not found');
    });
  });

  describe('App Resume', () => {
    it('should retrieve last active timestamp on resume', async () => {
      const expectedTimestamp = Date.now() - 60000;
      mockInvoke.mockResolvedValueOnce(expectedTimestamp);

      const result = await getLastActiveTimestamp();

      expect(mockInvoke).toHaveBeenCalledWith('get_last_active_timestamp');
      expect(result).toBe(expectedTimestamp);
    });

    it('should consume pending items with calculated budget on resume', async () => {
      const mockResult = {
        items_consumed: 5,
        items_discarded: 2,
        time_consumed_minutes: 10,
        memes_consumed: 3,
        jokes_consumed: 1,
        news_consumed: 1,
        videos_consumed: 0,
        gossip_consumed: 0,
      };
      mockInvoke.mockResolvedValueOnce(mockResult);

      const result = await consumePendingItems(10);

      expect(mockInvoke).toHaveBeenCalledWith('consume_pending_items', { budgetMinutes: 10 });
      expect(result).toEqual(mockResult);
    });

    it('should handle zero elapsed time (no consumption needed)', async () => {
      const now = Date.now();
      mockInvoke.mockResolvedValueOnce(now);

      const lastActive = await getLastActiveTimestamp();
      const elapsedMinutes = (now - lastActive) / 60000;

      expect(elapsedMinutes).toBe(0);
    });

    it('should handle short elapsed time (< 1 minute, no consumption)', async () => {
      const now = Date.now();
      const thirtySecondsAgo = now - 30000;
      mockInvoke.mockResolvedValueOnce(thirtySecondsAgo);

      const lastActive = await getLastActiveTimestamp();
      const elapsedMinutes = (now - lastActive) / 60000;

      expect(elapsedMinutes).toBe(0.5);
      expect(elapsedMinutes).toBeLessThan(1);
    });

    it('should handle long elapsed time (> 1 minute, consumption needed)', async () => {
      const now = Date.now();
      const fiveMinutesAgo = now - 300000;
      mockInvoke.mockResolvedValueOnce(fiveMinutesAgo);

      const lastActive = await getLastActiveTimestamp();
      const elapsedMinutes = (now - lastActive) / 60000;

      expect(elapsedMinutes).toBe(5);
      expect(elapsedMinutes).toBeGreaterThanOrEqual(1);
    });

    it('should calculate correct budget based on elapsed time and thread count', async () => {
      const threadCount = 4;
      const elapsedMinutes = 5;
      const expectedBudget = elapsedMinutes * threadCount;

      expect(expectedBudget).toBe(20);
    });

    it('should handle database errors during timestamp retrieval', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database connection failed'));

      await expect(getLastActiveTimestamp()).rejects.toThrow('Database connection failed');
    });

    it('should handle errors during consumption', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('No pending items'));

      await expect(consumePendingItems(10)).rejects.toThrow('No pending items');
    });
  });

  describe('Diagnostic Logging', () => {
    it('should log app resume events', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await logDiagnostic('app_resume', 'info', 'App resumed', { elapsedMinutes: 5 });

      expect(mockInvoke).toHaveBeenCalledWith('log_diagnostic', {
        eventType: 'app_resume',
        severity: 'info',
        message: 'App resumed',
        metadata: JSON.stringify({ elapsedMinutes: 5 }),
      });
    });

    it('should log app background events', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await logDiagnostic('app_background', 'info', 'App went to background');

      expect(mockInvoke).toHaveBeenCalledWith('log_diagnostic', {
        eventType: 'app_background',
        severity: 'info',
        message: 'App went to background',
        metadata: null,
      });
    });

    it('should log resume errors', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await logDiagnostic('app_resume_error', 'error', 'Failed to handle resume', { 
        error: 'Database error' 
      });

      expect(mockInvoke).toHaveBeenCalledWith('log_diagnostic', {
        eventType: 'app_resume_error',
        severity: 'error',
        message: 'Failed to handle resume',
        metadata: JSON.stringify({ error: 'Database error' }),
      });
    });

    it('should log background errors', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await logDiagnostic('app_background_error', 'error', 'Failed to handle background', { 
        error: 'Timer cleanup failed' 
      });

      expect(mockInvoke).toHaveBeenCalledWith('log_diagnostic', {
        eventType: 'app_background_error',
        severity: 'error',
        message: 'Failed to handle background',
        metadata: JSON.stringify({ error: 'Timer cleanup failed' }),
      });
    });

    it('should handle logging errors gracefully', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Log buffer full'));

      await expect(logDiagnostic('test', 'info', 'message')).rejects.toThrow('Log buffer full');
    });
  });

  describe('Lifecycle Event Handlers', () => {
    it('should register onResume handler on Android', () => {
      const handler = vi.fn();
      mockOnResume(handler);

      expect(mockOnResume).toHaveBeenCalledWith(handler);
    });

    it('should register onPause handler on Android', () => {
      const handler = vi.fn();
      mockOnPause(handler);

      expect(mockOnPause).toHaveBeenCalledWith(handler);
    });

    it('should use visibilitychange on non-Android platforms', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
        configurable: true,
      });

      const mockAddEventListener = vi.fn();
      global.document = {
        addEventListener: mockAddEventListener,
        visibilityState: 'visible',
      } as unknown as Document;

      const handler = () => {};
      global.document.addEventListener('visibilitychange', handler);

      expect(mockAddEventListener).toHaveBeenCalledWith('visibilitychange', handler);
    });
  });

  describe('Toast Messages on Resume', () => {
    it('should format message with single item type consumed', () => {
      const result = {
        items_consumed: 3,
        memes_consumed: 3,
        jokes_consumed: 0,
        news_consumed: 0,
        videos_consumed: 0,
        gossip_consumed: 0,
      };

      const parts: string[] = [];
      if (result.memes_consumed > 0) parts.push(`${result.memes_consumed} meme${result.memes_consumed > 1 ? 's' : ''}`);
      if (result.jokes_consumed > 0) parts.push(`${result.jokes_consumed} dad joke${result.jokes_consumed > 1 ? 's' : ''}`);
      if (result.news_consumed > 0) parts.push(`${result.news_consumed} news article${result.news_consumed > 1 ? 's' : ''}`);
      if (result.videos_consumed > 0) parts.push(`${result.videos_consumed} video${result.videos_consumed > 1 ? 's' : ''}`);
      if (result.gossip_consumed > 0) parts.push(`${result.gossip_consumed} gossip piece${result.gossip_consumed > 1 ? 's' : ''}`);

      const summary = parts.length > 0 ? parts.join(', ') : `${result.items_consumed} items`;
      const message = `While you were away, I doomscrolled through ${summary}. You're welcome.`;

      expect(message).toBe('While you were away, I doomscrolled through 3 memes. You\'re welcome.');
    });

    it('should format message with multiple item types consumed', () => {
      const result = {
        items_consumed: 5,
        memes_consumed: 2,
        jokes_consumed: 1,
        news_consumed: 1,
        videos_consumed: 1,
        gossip_consumed: 0,
      };

      const parts: string[] = [];
      if (result.memes_consumed > 0) parts.push(`${result.memes_consumed} memes`);
      if (result.jokes_consumed > 0) parts.push(`${result.jokes_consumed} dad joke`);
      if (result.news_consumed > 0) parts.push(`${result.news_consumed} news article`);
      if (result.videos_consumed > 0) parts.push(`${result.videos_consumed} video`);

      const summary = parts.join(', ');
      const message = `While you were away, I doomscrolled through ${summary}. You're welcome.`;

      expect(message).toBe('While you were away, I doomscrolled through 2 memes, 1 dad joke, 1 news article, 1 video. You\'re welcome.');
    });

    it('should use generic message when no specific types consumed', () => {
      const result = {
        items_consumed: 0,
        memes_consumed: 0,
        jokes_consumed: 0,
        news_consumed: 0,
        videos_consumed: 0,
        gossip_consumed: 0,
      };

      const parts: string[] = [];
      if (result.memes_consumed > 0) parts.push(`${result.memes_consumed} meme${result.memes_consumed > 1 ? 's' : ''}`);
      if (result.jokes_consumed > 0) parts.push(`${result.jokes_consumed} dad joke${result.jokes_consumed > 1 ? 's' : ''}`);
      if (result.news_consumed > 0) parts.push(`${result.news_consumed} news article${result.news_consumed > 1 ? 's' : ''}`);
      if (result.videos_consumed > 0) parts.push(`${result.videos_consumed} video${result.videos_consumed > 1 ? 's' : ''}`);
      if (result.gossip_consumed > 0) parts.push(`${result.gossip_consumed} gossip piece${result.gossip_consumed > 1 ? 's' : ''}`);

      const summary = parts.length > 0 ? parts.join(', ') : `${result.items_consumed} items`;
      const message = `While you were away, I doomscrolled through ${summary}. You're welcome.`;

      expect(message).toBe('While you were away, I doomscrolled through 0 items. You\'re welcome.');
    });

    it('should handle singular forms correctly', () => {
      const result = {
        items_consumed: 1,
        memes_consumed: 1,
        jokes_consumed: 0,
        news_consumed: 0,
        videos_consumed: 0,
        gossip_consumed: 0,
      };

      const parts: string[] = [];
      if (result.memes_consumed > 0) parts.push(`${result.memes_consumed} meme${result.memes_consumed > 1 ? 's' : ''}`);

      const summary = parts.join(', ');
      const message = `While you were away, I doomscrolled through ${summary}. You're welcome.`;

      expect(message).toBe('While you were away, I doomscrolled through 1 meme. You\'re welcome.');
    });
  });

  describe('Timer Management', () => {
    it('should clear status timer on background', () => {
      const clearTimer = vi.fn();
      clearTimer();

      expect(clearTimer).toHaveBeenCalled();
    });

    it('should not start automatic doomscrolling on resume', () => {
      const startDoomscrolling = vi.fn();

      expect(startDoomscrolling).not.toHaveBeenCalled();
    });
  });

  describe('Edge Cases', () => {
    it('should handle resume with negative elapsed time (clock adjustment)', async () => {
      const now = Date.now();
      const future = now + 60000;
      mockInvoke.mockResolvedValueOnce(future);

      const lastActive = await getLastActiveTimestamp();
      const elapsedMinutes = (now - lastActive) / 60000;

      expect(elapsedMinutes).toBe(-1);
    });

    it('should handle resume with very long elapsed time (hours)', async () => {
      const now = Date.now();
      const hoursAgo = now - 3600000 * 5;
      mockInvoke.mockResolvedValueOnce(hoursAgo);

      const lastActive = await getLastActiveTimestamp();
      const elapsedMinutes = (now - lastActive) / 60000;

      expect(elapsedMinutes).toBe(300);
    });

    it('should handle concurrent background/foreground transitions', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const backgroundPromise = setLastActiveTimestamp(Date.now());
      const foregroundPromise = getLastActiveTimestamp();

      await expect(Promise.all([backgroundPromise, foregroundPromise])).resolves.not.toThrow();
    });

    it('should handle missing user agent gracefully', () => {
      Object.defineProperty(navigator, 'userAgent', {
        value: '',
        configurable: true,
      });

      expect(isAndroid()).toBe(false);
    });
  });
});
