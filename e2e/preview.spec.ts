import { test, expect } from '@playwright/test';
import { getTestStoreState } from './fixtures/crawlItems';

test.describe('Hover Preview Functionality', () => {
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

  test('should display meme preview with image, title, and source', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const item = items.filter({ has: page.locator('.text-\\[10px\\]:text-matches("MEME", "i")') }).first();
    
    await item.hover();
    await page.waitForTimeout(500);
    
    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    
    await expect(preview.locator('text=/CAT:MEME/i')).toBeVisible();
    
    const previewImg = preview.locator('img');
    await expect(previewImg).toBeVisible();
    
    await expect(preview.locator('text=/SRC:/i')).toBeVisible();
  });

  test('should display joke preview with text content and source', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const item = items.filter({ has: page.locator('.text-\\[10px\\]:text-matches("JOKE", "i")') }).first();
    
    await item.hover();
    await page.waitForTimeout(500);
    
    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    
    await expect(preview.locator('text=/CAT:JOKE/i')).toBeVisible();
    
    await expect(preview.locator('p.italic')).toBeVisible();
    
    await expect(preview.locator('text=/SRC:/i')).toBeVisible();
  });

  test('should display news preview with image, title, description, and source', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const item = items.filter({ has: page.locator('.text-\\[10px\\]:text-matches("NEWS", "i")') }).first();
    
    await item.hover();
    await page.waitForTimeout(500);
    
    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    
    await expect(preview.locator('text=/CAT:NEWS/i')).toBeVisible();
    
    const previewImg = preview.locator('img');
    await expect(previewImg).toBeVisible();
    
    await expect(preview.locator('h3.font-bold')).toBeVisible();
    
    await expect(preview.locator('text=/SRC:/i')).toBeVisible();
  });

  test('should display video preview with thumbnail, title, and source', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const item = items.filter({ has: page.locator('.text-\\[10px\\]:text-matches("VIDEO", "i")') }).first();
    
    await item.hover();
    await page.waitForTimeout(500);
    
    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    
    await expect(preview.locator('text=/CAT:VIDEO/i')).toBeVisible();
    
    const previewImg = preview.locator('img');
    await expect(previewImg).toBeVisible();
    
    await expect(preview.locator('p.font-bold')).toBeVisible();
    
    await expect(preview.locator('text=/SRC:/i')).toBeVisible();
  });

  test('should display gossip preview with title and source', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const item = items.filter({ has: page.locator('.text-\\[10px\\]:text-matches("GOSSIP", "i")') }).first();
    
    await item.hover();
    await page.waitForTimeout(500);
    
    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    
    await expect(preview.locator('text=/CAT:GOSSIP/i')).toBeVisible();
    
    const titleOrIntel = preview.locator('p.text-lg.font-bold.uppercase');
    await expect(titleOrIntel).toBeVisible();
    
    await expect(preview.locator('text=/SRC:/i')).toBeVisible();
  });

  test('should close preview when mouse leaves item area', async ({ page }) => {
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

  test('should keep preview open when hovering into preview area', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const firstItem = items.first();
    await firstItem.hover();
    await page.waitForTimeout(500);

    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();

    await preview.hover();
    await page.waitForTimeout(300);

    await expect(preview).toBeVisible();

    await page.mouse.move(10, 10);
    await page.waitForTimeout(300);

    await expect(preview).not.toBeVisible();
  });

  test('should close preview when mouse leaves preview area to empty space', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const firstItem = items.first();
    
    await firstItem.hover();
    await page.waitForTimeout(500);
    
    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    
    await preview.hover();
    await page.waitForTimeout(300);
    await expect(preview).toBeVisible();
    
    await page.mouse.move(10, 10);
    await page.waitForTimeout(500);
    
    await expect(preview).not.toBeVisible();
  });

  test('should update preview when moving from one item to another', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');

    const firstItem = items.first();
    await firstItem.hover();
    await page.waitForTimeout(500);

    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    await expect(preview.locator('text=/CAT:MEME/i')).toBeVisible();

    const firstBox = await firstItem.boundingBox();
    const secondItem = items.nth(1);
    const secondBox = await secondItem.boundingBox();
    
    if (firstBox && secondBox) {
      const gapX = firstBox.x + firstBox.width / 2;
      const gapY = firstBox.y + firstBox.height + 2;
      await page.mouse.move(gapX, gapY);
      await page.waitForTimeout(250);
      
      await secondItem.hover();
      await page.waitForTimeout(300);
    } else {
      await secondItem.hover();
      await page.waitForTimeout(300);
    }

    await expect(preview).toBeVisible();
    await expect(preview.locator('text=/CAT:JOKE/i')).toBeVisible();
  });

  test('should display category label and item ID in preview', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const firstItem = items.first();
    await firstItem.hover();
    await page.waitForTimeout(500);
    
    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    
    await expect(preview.locator('text=/\\[PREVIEW_TERMINAL \\/\\/ CAT:/i')).toBeVisible();
    await expect(preview.locator('text=/ID:/i')).toBeVisible();
  });

  test('should display click to open prompt in preview', async ({ page }) => {
    const items = page.locator('.flex.gap-4.p-3');
    const firstItem = items.first();
    await firstItem.hover();
    await page.waitForTimeout(500);
    
    const preview = page.locator('.fixed.inset-x-0.bottom-0');
    await expect(preview).toBeVisible();
    
    await expect(preview.locator('text=/CLICK TO EXECUTE_OPEN/i')).toBeVisible();
  });
});
