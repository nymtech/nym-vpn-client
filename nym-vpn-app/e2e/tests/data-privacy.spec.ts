import { expect, test } from '@playwright/test';

const privacySaveError = "Couldn't save your privacy choice. Try again.";

test.describe('Data and privacy telemetry', () => {
  test('keeps the previous network stats choice when the save fails', async ({
    page,
  }) => {
    await page.addInitScript(() => {
      (window as Window & { __NYM_IPC_FAIL?: string[] }).__NYM_IPC_FAIL = [
        'disable_netstats',
      ];
    });
    await page.goto('/settings/data-privacy');
    await page.waitForLoadState('networkidle');

    const netstats = page.getByTestId('data-privacy-netstats-switch');
    await expect(netstats).toHaveAttribute('aria-checked', 'true');
    await netstats.click();

    await expect(netstats).toHaveAttribute('aria-checked', 'true');
    await expect(page.getByText(privacySaveError)).toBeVisible();
  });

  test('updates the network stats switch after a successful save', async ({
    page,
  }) => {
    await page.goto('/settings/data-privacy');
    await page.waitForLoadState('networkidle');

    const netstats = page.getByTestId('data-privacy-netstats-switch');
    await expect(netstats).toHaveAttribute('aria-checked', 'true');
    await netstats.click();

    await expect(netstats).toHaveAttribute('aria-checked', 'false');
  });
});
