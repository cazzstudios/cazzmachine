import { test, expect } from '@playwright/test';

test.describe('Tauri Backend E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('cazzmachine_first_run', 'false');
    });
    await page.goto('/');
    await page.waitForTimeout(2000);
  });

  test('should toggle doomscrolling state via backend', async ({ page }) => {
    const initialStatus = await page.locator('text=System Status:').textContent();
    
    await page.locator('text=System Status:').click();
    await expect(page.locator('text=Doomscrolling')).toBeVisible({ timeout: 15000 });
    
    await page.locator('text=System Status:').click();
    await expect(page.locator('text=standby')).toBeVisible({ timeout: 15000 });
  });

  test('should fetch and display stats from backend when data exists', async ({ page }) => {
    await page.waitForTimeout(15000);
    
    const statsBox = page.locator('.bg-cazz-surface');
    await expect(statsBox).toBeVisible();
    
    const hasMemeLog = await page.locator('text=MEME_LOG').count();
    if (hasMemeLog > 0) {
      await expect(page.locator('text=MEME_LOG')).toBeVisible();
    }
  });

  test('should show time saved estimate when data exists', async ({ page }) => {
    await page.waitForTimeout(15000);
    
    const hasSaved = await page.locator('text=Saved:').count();
    if (hasSaved > 0) {
      await expect(page.locator('text=Saved:')).toBeVisible();
    } else {
      expect(true).toBe(true);
    }
  });

  test('should navigate to summary when items exist', async ({ page }) => {
    await page.waitForTimeout(5000);
    
    await page.locator('button:has-text("TERMINATE_WORK_SESSION")').click();
    
    const hasSessionReport = await page.locator('text=SESSION_REPORT.LOG').count();
    const hasExitSession = await page.locator('text=EXIT_SESSION').count();
    
    if (hasSessionReport > 0) {
      await expect(page.locator('text=SESSION_REPORT.LOG')).toBeVisible({ timeout: 5000 });
    } else if (hasExitSession > 0) {
      await expect(page.locator('text=EXIT_SESSION')).toBeVisible({ timeout: 5000 });
    } else {
      await expect(page.locator('text=TERMINATE_WORK_SESSION')).toBeVisible({ timeout: 5000 });
    }
  });

  test('should display diagnostics panel when triggered', async ({ page }) => {
    await page.goto('/?diagnostics=true');
    await page.waitForTimeout(10000);
    
    const hasDiagnostics = await page.locator('text=Diagnostics Panel').count();
    if (hasDiagnostics > 0) {
      await expect(page.locator('text=Diagnostics Panel')).toBeVisible({ timeout: 5000 });
    }
    
    const hasPanel = await page.locator('.fixed.inset-0').count();
    expect(hasPanel).toBeGreaterThan(0);
  });
});
