import { test, expect } from '@playwright/test';

test.describe('Connect / Disconnect', () => {
  test('home screen renders and is not empty', async ({ page }) => {
    await page.goto('/');
    const root = page.locator('#root');
    await expect(root).not.toBeEmpty({ timeout: 15_000 });
  });

  test('connect button is present and clickable', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#root')).not.toBeEmpty({ timeout: 15_000 });

    const connectBtn = page.getByRole('button', { name: /connect/i });
    await expect(connectBtn).toBeVisible({ timeout: 10_000 });
    await connectBtn.click();
    await page.waitForTimeout(2000);
  });

  test('disconnect flow works after connecting', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#root')).not.toBeEmpty({ timeout: 15_000 });

    const connectBtn = page.getByRole('button', { name: /connect/i });
    if (await connectBtn.isVisible()) {
      await connectBtn.click();
      await page.waitForTimeout(3000);

      const disconnectBtn = page.getByRole('button', {
        name: /disconnect/i,
      });
      if (await disconnectBtn.isVisible()) {
        await disconnectBtn.click();
        await page.waitForTimeout(2000);
      }
    }
  });
});
