import { describe, it, expect } from 'vitest';
import type { ConsumeResult } from '../src/lib/tauri';
import {
  getDoomscrollDurationMs,
  getThreadDurations,
  generatePhaseEndToast,
} from '../src/stores/appStore';

describe('getDoomscrollDurationMs', () => {
  it('should return 1 minute (60000 ms) for level 1', () => {
    const result = getDoomscrollDurationMs(1);
    expect(result).toBe(60000);
  });

  it('should return 5 minutes (300000 ms) for level 9', () => {
    const result = getDoomscrollDurationMs(9);
    expect(result).toBe(300000);
  });

  it('should follow the formula: 1 + 4 * ((level - 1) / 8) minutes', () => {
    // Level 1: 1 + 4 * (0/8) = 1 min = 60000 ms
    expect(getDoomscrollDurationMs(1)).toBe(60000);
    
    // Level 5: 1 + 4 * (4/8) = 1 + 2 = 3 min = 180000 ms
    expect(getDoomscrollDurationMs(5)).toBe(180000);
    
    // Level 9: 1 + 4 * (8/8) = 1 + 4 = 5 min = 300000 ms
    expect(getDoomscrollDurationMs(9)).toBe(300000);
  });

  it('should return linearly interpolated values for intermediate levels', () => {
    // Level 3: 1 + 4 * (2/8) = 1 + 1 = 2 min = 120000 ms
    expect(getDoomscrollDurationMs(3)).toBe(120000);
    
    // Level 7: 1 + 4 * (6/8) = 1 + 3 = 4 min = 240000 ms
    expect(getDoomscrollDurationMs(7)).toBe(240000);
  });
});

describe('getThreadDurations', () => {
  it('should return [budgetMs] when threadCount is 1', () => {
    const budgetMs = 300000; // 5 minutes
    const result = getThreadDurations(budgetMs, 1);
    expect(result).toEqual([300000]);
  });

  it('should return array with correct length for threadCount > 1', () => {
    const budgetMs = 300000;
    
    expect(getThreadDurations(budgetMs, 2)).toHaveLength(2);
    expect(getThreadDurations(budgetMs, 4)).toHaveLength(4);
    expect(getThreadDurations(budgetMs, 8)).toHaveLength(8);
  });

  it('should distribute values from half budget to full budget', () => {
    const budgetMs = 300000; // 5 minutes
    const threadCount = 3;
    const result = getThreadDurations(budgetMs, threadCount);
    
    const half = budgetMs / 2; // 150000
    
    // For 3 threads: 
    // i=0: 150000 + (0 * 150000) / 2 = 150000
    // i=1: 150000 + (1 * 150000) / 2 = 225000
    // i=2: 150000 + (2 * 150000) / 2 = 300000
    expect(result[0]).toBe(150000);
    expect(result[1]).toBe(225000);
    expect(result[2]).toBe(300000);
  });

  it('should have last value equal to full budget', () => {
    const budgetMs = 180000;
    
    const result2 = getThreadDurations(budgetMs, 2);
    expect(result2[result2.length - 1]).toBe(budgetMs);
    
    const result5 = getThreadDurations(budgetMs, 5);
    expect(result5[result5.length - 1]).toBe(budgetMs);
  });

  it('should have first value equal to half budget when threadCount > 1', () => {
    const budgetMs = 200000;
    
    const result3 = getThreadDurations(budgetMs, 3);
    expect(result3[0]).toBe(100000);
    
    const result4 = getThreadDurations(budgetMs, 4);
    expect(result4[0]).toBe(100000);
  });
});

describe('generatePhaseEndToast', () => {
  it('should return "No content found" message when items_consumed is 0', () => {
    const result: ConsumeResult = {
      items_consumed: 0,
      items_discarded: 0,
      time_consumed_minutes: 0,
      memes_consumed: 0,
      jokes_consumed: 0,
      news_consumed: 0,
      videos_consumed: 0,
      gossip_consumed: 0,
    };
    
    const message = generatePhaseEndToast(result);
    expect(message).toBe("No content found. The internet is quiet... suspiciously quiet.");
  });

  it('should return string with content when items_consumed > 0', () => {
    const result: ConsumeResult = {
      items_consumed: 5,
      items_discarded: 0,
      time_consumed_minutes: 10,
      memes_consumed: 3,
      jokes_consumed: 2,
      news_consumed: 0,
      videos_consumed: 0,
      gossip_consumed: 0,
    };
    
    const message = generatePhaseEndToast(result);
    expect(message).toBeTruthy();
    expect(message).not.toBe("No content found. The internet is quiet... suspiciously quiet.");
    expect(message.length).toBeGreaterThan(0);
  });

  it('should include meme count in message when memes_consumed > 0', () => {
    const result: ConsumeResult = {
      items_consumed: 3,
      items_discarded: 0,
      time_consumed_minutes: 5,
      memes_consumed: 3,
      jokes_consumed: 0,
      news_consumed: 0,
      videos_consumed: 0,
      gossip_consumed: 0,
    };
    
    const message = generatePhaseEndToast(result);
    expect(message).toContain("3 memes");
  });

  it('should pluralize correctly for single items', () => {
    const result: ConsumeResult = {
      items_consumed: 1,
      items_discarded: 0,
      time_consumed_minutes: 2,
      memes_consumed: 1,
      jokes_consumed: 1,
      news_consumed: 1,
      videos_consumed: 1,
      gossip_consumed: 1,
    };
    
    const message = generatePhaseEndToast(result);
    expect(message).toContain("1 meme");
    expect(message).toContain("1 dad joke");
    expect(message).toContain("1 news article");
    expect(message).toContain("1 video");
    expect(message).toContain("1 gossip piece");
  });

  it('should pluralize correctly for multiple items', () => {
    const result: ConsumeResult = {
      items_consumed: 10,
      items_discarded: 0,
      time_consumed_minutes: 15,
      memes_consumed: 2,
      jokes_consumed: 2,
      news_consumed: 2,
      videos_consumed: 2,
      gossip_consumed: 2,
    };
    
    const message = generatePhaseEndToast(result);
    expect(message).toContain("2 memes");
    expect(message).toContain("2 dad jokes");
    expect(message).toContain("2 news articles");
    expect(message).toContain("2 videos");
    expect(message).toContain("2 gossip pieces");
  });

  it('should include all content types in message when all are consumed', () => {
    const result: ConsumeResult = {
      items_consumed: 15,
      items_discarded: 0,
      time_consumed_minutes: 20,
      memes_consumed: 3,
      jokes_consumed: 4,
      news_consumed: 2,
      videos_consumed: 1,
      gossip_consumed: 5,
    };
    
    const message = generatePhaseEndToast(result);
    expect(message).toContain("3 memes");
    expect(message).toContain("4 dad jokes");
    expect(message).toContain("2 news articles");
    expect(message).toContain("1 video");
    expect(message).toContain("5 gossip pieces");
  });

  it('should use generic "items" message when no specific types are consumed', () => {
    const result: ConsumeResult = {
      items_consumed: 5,
      items_discarded: 0,
      time_consumed_minutes: 10,
      memes_consumed: 0,
      jokes_consumed: 0,
      news_consumed: 0,
      videos_consumed: 0,
      gossip_consumed: 0,
    };
    
    const message = generatePhaseEndToast(result);
    expect(message).toContain("5 items");
    expect(message).not.toContain("meme");
    expect(message).not.toContain("joke");
    expect(message).not.toContain("article");
    expect(message).not.toContain("video");
    expect(message).not.toContain("gossip");
  });

  it('should return one of the predefined templates', () => {
    const result: ConsumeResult = {
      items_consumed: 3,
      items_discarded: 0,
      time_consumed_minutes: 5,
      memes_consumed: 3,
      jokes_consumed: 0,
      news_consumed: 0,
      videos_consumed: 0,
      gossip_consumed: 0,
    };
    
    // Run multiple times to check we get valid templates
    const templates = [
      /Doomscrolled .* so you don't have to/,
      /Just inhaled .*/,
      /Consumed .*/,
      /.* catalogued\. Your procrastination proxy delivers/,
      /Finished binging .*/,
      /.* distractions neutralized/,
      /Another haul:/,
      /.* — consumed and catalogued\. The Cazzmachine delivers/,
      /Swallowed .* whole/,
      /That's .* you don't have to think about/,
      /.* items down the hatch/,
      /Just powered through .*/,
      /Inhaled .* in record time/,
      /Another cycle done:/,
      /.* All consumed\. All catalogued\. All pointless/,
      /Dispatched .* with surgical efficiency/,
      /.* distractions intercepted before they reached you/,
      /Fed on .*/,
      /Burned through .*/,
      /Just binged .*/,
      /.* — probably half AI-generated/,
      /Ingested .*/,
    ]
    
    const message = generatePhaseEndToast(result);
    const matchesTemplate = templates.some(pattern => pattern.test(message));
    expect(matchesTemplate).toBe(true);
  });
});
