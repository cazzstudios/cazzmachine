import { test, expect } from '@playwright/test';

test.describe('Tauri Functionality', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should load without critical console errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.waitForTimeout(2000);
    
    const criticalErrors = errors.filter(e => 
      !e.includes('warning') && !e.includes('net::ERR')
    );
    expect(criticalErrors).toHaveLength(0);
  });

  test('should display status box', async ({ page }) => {
    const statusBox = page.locator('.rounded-xl.p-4.w-full');
    await expect(statusBox).toBeVisible();
  });
});
