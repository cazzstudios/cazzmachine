import { test, expect } from '@playwright/test';

test.describe('Cazzmachine App', () => {
  test.beforeEach(async ({ page }) => {
    await page.context().addInitScript(() => {
      localStorage.setItem("cazzmachine_first_run", "false");
    });
    await page.goto('/');
  });

  test('should load the main page', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('cazzmachine');
  });

  test('should display the app logo and heading', async ({ page }) => {
    const logo = page.locator('img[alt="doomscrolling"]');
    await expect(logo).toBeVisible();
    
    const heading = page.locator('h1:has-text("cazzmachine")');
    await expect(heading).toBeVisible();
  });

  test('should display system status', async ({ page }) => {
    const statusElement = page.locator('text=System Status:');
    await expect(statusElement).toBeVisible();
  });

  test('should display throttle knob control', async ({ page }) => {
    const throttleSection = page.locator('text=Controls');
    await expect(throttleSection).toBeVisible();
  });

  test('should display terminate work session button', async ({ page }) => {
    const terminateButton = page.locator('button:has-text("TERMINATE_WORK_SESSION")');
    await expect(terminateButton).toBeVisible();
  });

  test('should display stealth preview mode link', async ({ page }) => {
    const previewLink = page.locator('text=Enter Stealth Preview Mode');
    await expect(previewLink).toBeVisible();
  });

  test('should start doomscrolling on status click when idle', async ({ page }) => {
    const statusElement = page.locator('text=System Status: standby');
    await expect(statusElement).toBeVisible();
    
    await page.locator('text=System Status:').click();
    
    await expect(page.locator('text=Doomscrolling')).toBeVisible({ timeout: 10000 });
  });

  test('should stop doomscrolling when clicked while scrolling', async ({ page }) => {
    await page.locator('text=System Status:').click();
    
    await expect(page.locator('text=Doomscrolling')).toBeVisible({ timeout: 10000 });
    
    await page.locator('text=System Status:').click();
    
    await expect(page.locator('text=standby')).toBeVisible({ timeout: 10000 });
  });

  test('should navigate to detail view via stealth preview', async ({ page }) => {
    await page.locator('text=Enter Stealth Preview Mode').click();
    
    await expect(page.locator('text=DATA_DUMP.LOG')).toBeVisible();
    await expect(page.locator('text=ESC_TERMINAL')).toBeVisible();
  });

  test('should display thread control with buttons', async ({ page }) => {
    const minusButton = page.locator('button:has-text("−")');
    const plusButton = page.locator('button:has-text("+")');
    await expect(minusButton).toBeVisible();
    await expect(plusButton).toBeVisible();
  });
});
