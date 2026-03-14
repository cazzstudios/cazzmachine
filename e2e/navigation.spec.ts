import { test, expect } from '@playwright/test';

test.describe('Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await page.context().addInitScript(() => {
      localStorage.setItem("cazzmachine_first_run", "false");
    });
    await page.goto('/');
  });

  test('should start on idle view', async ({ page }) => {
    await expect(page.locator('h1:has-text("cazzmachine")')).toBeVisible();
    await expect(page.locator('text=la macchina che cazzeggia per te')).toBeVisible();
  });

  test('should navigate from idle to detail view', async ({ page }) => {
    await page.locator('text=Enter Stealth Preview Mode').click();
    
    await expect(page.locator('text=DATA_DUMP.LOG')).toBeVisible();
    await expect(page.locator('text=ESC_TERMINAL')).toBeVisible();
  });

  test('should navigate from detail view back to idle', async ({ page }) => {
    await page.locator('text=Enter Stealth Preview Mode').click();
    await expect(page.locator('text=DATA_DUMP.LOG')).toBeVisible();
    
    await page.locator('text=ESC_TERMINAL').click();
    
    await expect(page.locator('h1:has-text("cazzmachine")')).toBeVisible();
  });
});
