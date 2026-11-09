import { expect, test } from '@playwright/test';

/**
 * Technical telemetry opt-in screen.
 *
 * The original bug (#6984) was that each card's switch was bound to the *other*
 * toggle's state. These tests lock the label-to-switch pairing at the DOM level
 * so a future re-wiring can't silently pass, and check that each info (i) button
 * opens its matching "What's shared" dialog.
 */
test.describe('Technical opt-in', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/technical-optin');
    await page.waitForLoadState('networkidle');
  });

  test('pairs each label with its own toggle', async ({ page }) => {
    const connection = page.getByTestId('welcome-netstats-option');
    await expect(connection.getByText('Your connection quality')).toBeVisible();
    await expect(
      connection.getByTestId('welcome-netstats-switch'),
    ).toBeVisible();

    const errors = page.getByTestId('welcome-monitoring-option');
    await expect(errors.getByText('Your app errors & crashes')).toBeVisible();
    await expect(errors.getByTestId('welcome-monitoring-switch')).toBeVisible();
  });

  test('toggling one switch leaves the other unchanged', async ({ page }) => {
    const netstats = page.getByTestId('welcome-netstats-switch');
    const monitoring = page.getByTestId('welcome-monitoring-switch');

    const netBefore = await netstats.getAttribute('aria-checked');
    const monBefore = await monitoring.getAttribute('aria-checked');

    await netstats.click();

    await expect(netstats).not.toHaveAttribute(
      'aria-checked',
      netBefore ?? 'false',
    );
    await expect(monitoring).toHaveAttribute(
      'aria-checked',
      monBefore ?? 'false',
    );
  });

  test('each info button opens its matching "What\'s shared" dialog', async ({
    page,
  }) => {
    const dialog = page.getByTestId('telemetry-info-dialog');

    await page.getByTestId('welcome-netstats-info').click();
    await expect(dialog.getByText("What's shared")).toBeVisible();
    await expect(
      dialog.getByText('Mode (dVPN / Mixnet) and exit server'),
    ).toBeVisible();
    await expect(
      dialog.getByText('Sent through the VPN tunnel to Nym. No third party.'),
    ).toBeVisible();
    await dialog.getByRole('button', { name: 'Got it' }).click();

    await page.getByTestId('welcome-monitoring-info').click();
    await expect(
      dialog.getByText(
        'Handled by Sentry, a third-party service. Nym never sees your IP.',
      ),
    ).toBeVisible();
  });
});
