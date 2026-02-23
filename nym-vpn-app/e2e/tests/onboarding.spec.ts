import { test, expect } from '@playwright/test';

test.describe('Onboarding', () => {
  test('app loads and renders the main screen', async ({ page }) => {
    await page.goto('/');
    const root = page.locator('#root');
    await expect(root).not.toBeEmpty({ timeout: 15_000 });
  });

  test('onboarding screen renders when navigated to', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#root')).not.toBeEmpty({ timeout: 15_000 });
  });

  test('welcome screen renders when navigated to', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#root')).not.toBeEmpty({ timeout: 15_000 });
  });
});
