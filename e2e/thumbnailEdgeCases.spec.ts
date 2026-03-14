import { test, expect } from '@playwright/test';
import { getTestStoreStateWithUrls } from './fixtures/crawlItems';

test.describe('Thumbnail Edge Cases', () => {
  test.beforeEach(async ({ page }) => {
    await page.context().addInitScript(() => {
      localStorage.setItem("cazzmachine_first_run", "false");
    });
    await page.goto('/');
    await page.waitForTimeout(500);
    await page.evaluate((state) => {
      (window as any).__APP_STORE__?.setState(state);
    }, getTestStoreStateWithUrls());
    await expect(page.locator('text=DATA_DUMP.LOG')).toBeVisible({ timeout: 5000 });
  });

  test('items with no thumbnail show no image container', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const noThumbItem = items.filter({ hasText: 'No Thumbnail Available' }).first();
    
    await expect(noThumbItem).toBeVisible();
    
    const imgInCard = noThumbItem.locator('img.w-full.h-full');
    await expect(imgInCard).not.toBeVisible();
  });

  test('preview shows content without image for items without thumbnail', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const noThumbItem = items.filter({ hasText: 'No Thumbnail Available' }).first();
    
    await noThumbItem.hover();
    await page.waitForTimeout(500);
    
    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    
    await expect(preview.locator('text=No Thumbnail Available')).toBeVisible();
    await expect(preview.locator('text=/SRC: test-gossip-feed/i')).toBeVisible();
    
    const previewImg = preview.locator('img');
    await expect(previewImg).not.toBeVisible();
  });

  test('preview closes when mouse leaves both item and preview areas', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const firstItem = items.first();
    await firstItem.hover();
    await page.waitForTimeout(500);
    
    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    
    await page.mouse.move(10, 10);
    await page.waitForTimeout(300);
    
    await expect(preview).not.toBeVisible();
  });

  test('preview stays open when moving from item to preview', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const firstItem = items.first();
    await firstItem.hover();
    await page.waitForTimeout(500);
    
    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    
    await preview.hover();
    await page.waitForTimeout(300);
    
    await expect(preview).toBeVisible();
  });
});
