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

    const disconnectBtn = page.getByRole('button', { name: /disconnect/i });
    await expect(disconnectBtn).toBeVisible({ timeout: 10_000 });
  });

  test('disconnect flow works after connecting', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#root')).not.toBeEmpty({ timeout: 15_000 });

    const connectBtn = page.getByRole('button', { name: /connect/i });
    await expect(connectBtn).toBeVisible({ timeout: 10_000 });
    await connectBtn.click();

    const disconnectBtn = page.getByRole('button', { name: /disconnect/i });
    await expect(disconnectBtn).toBeVisible({ timeout: 10_000 });
    await disconnectBtn.click();

    await expect(
      page.getByRole('button', { name: /connect/i }),
    ).toBeVisible({ timeout: 10_000 });
  });
});
