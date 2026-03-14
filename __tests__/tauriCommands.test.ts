import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the Tauri core module before importing
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import {
  getTodayItems,
  toggleSaveItem,
  consumePendingItems,
  triggerCrawl,
} from '../src/lib/tauri';

describe('Tauri Command Wrappers', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('getTodayItems', () => {
    it('should call invoke with correct command name', async () => {
      const mockItems = [
        {
          id: '1',
          source: 'reddit',
          category: 'meme',
          title: 'Test Item',
          url: 'https://example.com',
          thumbnail_url: null,
          description: null,
          fetched_at: '2024-01-01T00:00:00Z',
          is_seen: false,
          is_saved: false,
          session_date: '2024-01-01',
        },
      ];
      mockInvoke.mockResolvedValueOnce(mockItems);

      const result = await getTodayItems();

      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith('get_today_items');
      expect(result).toEqual(mockItems);
    });

    it('should return empty array when no items', async () => {
      mockInvoke.mockResolvedValueOnce([]);

      const result = await getTodayItems();

      expect(mockInvoke).toHaveBeenCalledWith('get_today_items');
      expect(result).toEqual([]);
    });
  });

  describe('toggleSaveItem', () => {
    it('should call invoke with correct command and itemId argument', async () => {
      const itemId = 'item-123';
      mockInvoke.mockResolvedValueOnce(true);

      const result = await toggleSaveItem(itemId);

      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith('toggle_save_item', { itemId });
      expect(result).toBe(true);
    });

    it('should pass different item IDs correctly', async () => {
      const itemId = 'abc-def-ghi';
      mockInvoke.mockResolvedValueOnce(false);

      await toggleSaveItem(itemId);

      expect(mockInvoke).toHaveBeenCalledWith('toggle_save_item', { itemId: 'abc-def-ghi' });
    });
  });

  describe('consumePendingItems', () => {
    it('should call invoke with correct command and budgetMinutes argument', async () => {
      const budgetMinutes = 5;
      const mockResult = {
        items_consumed: 3,
        items_discarded: 1,
        time_consumed_minutes: 4.5,
        memes_consumed: 2,
        jokes_consumed: 1,
        news_consumed: 0,
        videos_consumed: 0,
        gossip_consumed: 0,
      };
      mockInvoke.mockResolvedValueOnce(mockResult);

      const result = await consumePendingItems(budgetMinutes);

      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith('consume_pending_items', { budgetMinutes });
      expect(result).toEqual(mockResult);
    });

    it('should handle different budget values', async () => {
      mockInvoke.mockResolvedValueOnce({
        items_consumed: 0,
        items_discarded: 0,
        time_consumed_minutes: 0,
        memes_consumed: 0,
        jokes_consumed: 0,
        news_consumed: 0,
        videos_consumed: 0,
        gossip_consumed: 0,
      });

      await consumePendingItems(10);

      expect(mockInvoke).toHaveBeenCalledWith('consume_pending_items', { budgetMinutes: 10 });
    });
  });

  describe('triggerCrawl', () => {
    it('should call invoke with correct command name', async () => {
      mockInvoke.mockResolvedValueOnce(5);

      const result = await triggerCrawl();

      expect(mockInvoke).toHaveBeenCalledTimes(1);
      expect(mockInvoke).toHaveBeenCalledWith('trigger_crawl');
      expect(result).toBe(5);
    });

    it('should return number of items crawled', async () => {
      mockInvoke.mockResolvedValueOnce(0);

      const result = await triggerCrawl();

      expect(mockInvoke).toHaveBeenCalledWith('trigger_crawl');
      expect(result).toBe(0);
    });
  });
});
