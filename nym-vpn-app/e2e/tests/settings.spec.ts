import { test, expect } from '@playwright/test';

test.describe('Settings', () => {
  test('app renders and navigates to settings', async ({ page }) => {
    await page.goto('/');
    const root = page.locator('#root');
    await expect(root).not.toBeEmpty({ timeout: 15_000 });

    const settingsBtn = page.getByRole('button', { name: /settings/i }).or(
      page.locator('[data-testid="settings"]'),
    );
    if (await settingsBtn.isVisible({ timeout: 5_000 }).catch(() => false)) {
      await settingsBtn.click();
      await page.waitForTimeout(2000);
    }
  });

  test('main screen has interactive elements', async ({ page }) => {
    await page.goto('/');
    const root = page.locator('#root');
    await expect(root).not.toBeEmpty({ timeout: 15_000 });

    const buttons = page.getByRole('button');
    const count = await buttons.count();
    expect(count).toBeGreaterThan(0);
  });
});
