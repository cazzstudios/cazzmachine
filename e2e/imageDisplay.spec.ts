import { test, expect } from '@playwright/test';
import { getTestStoreState } from './fixtures/crawlItems';

test.describe('Image Display Verification', () => {
  test.beforeEach(async ({ page }) => {
    await page.context().addInitScript(() => {
      localStorage.setItem("cazzmachine_first_run", "false");
    });
    await page.goto('/');
    await page.waitForTimeout(500);
    
    await page.evaluate((state) => {
      (window as any).__APP_STORE__?.setState(state);
    }, getTestStoreState());
    
    await expect(page.locator('text=DATA_DUMP.LOG')).toBeVisible({ timeout: 5000 });
  });

  test('should display images in thumbnail and preview for items with images', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const itemsWithImages = items.filter({ has: page.locator('img.w-full.h-full') });
    
    const firstItemWithImage = itemsWithImages.first();
    const thumbnailImg = firstItemWithImage.locator('img.w-full.h-full');
    
    await expect(thumbnailImg).toBeVisible();
    
    await firstItemWithImage.hover();
    await page.waitForTimeout(500);
    
    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    
    const previewImg = preview.locator('img').first();
    await expect(previewImg).toBeVisible();
  });
});
