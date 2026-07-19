import { expect, test } from '@playwright/test';
import MainPage from '../pages/MainPage';

test.describe('Connection', () => {
  let mainPage: MainPage;

  test.beforeEach(async ({ page }) => {
    mainPage = new MainPage(page);
    await mainPage.goto();
  });

  test('shows the connecting state after tapping connect', async () => {
    await mainPage.connectButton.click();

    // While connecting the primary action becomes a cancel affordance and the
    // status label reports tunnel progress.
    await expect(mainPage.cancelButton).toBeVisible();
    await expect(mainPage.statusText).toBeVisible();
  });

  test('connects to VPN correctly', async () => {
    await expect(mainPage.statusText).toHaveText('Not protected');

    await mainPage.connectButton.click();

    await expect(mainPage.disconnectButton).toBeVisible({ timeout: 15_000 });
    await expect(mainPage.connectionTimer).toBeVisible();
    // The connected state replaces the status label with the connection timer.
    await expect(mainPage.statusText).toHaveCount(0);
  });

  test('resolves entry and exit servers once connected', async () => {
    await mainPage.connectButton.click();
    await expect(mainPage.disconnectButton).toBeVisible({ timeout: 15_000 });

    // Rows switch from the "Random" placeholder to the resolved gateways.
    await expect(mainPage.serverRow('Entry')).not.toContainText('Random');
    await expect(mainPage.serverRow('Exit')).not.toContainText('Random');
  });

  test('disconnects from VPN correctly', async () => {
    await mainPage.connectButton.click();
    await expect(mainPage.disconnectButton).toBeVisible({ timeout: 15_000 });

    await mainPage.disconnectButton.click();

    await expect(mainPage.connectButton).toBeVisible({ timeout: 15_000 });
    await expect(mainPage.statusText).toHaveText('Not protected');
    await expect(mainPage.connectionTimer).toHaveCount(0);
  });

  test('connects from different modes correctly', async () => {
    await mainPage.fastMode.click();
    await expect(mainPage.fastMode).toHaveAttribute('aria-pressed', 'true');
    await mainPage.connectButton.click();
    await expect(mainPage.disconnectButton).toBeVisible({ timeout: 15_000 });
    await mainPage.disconnectButton.click();
    await expect(mainPage.statusText).toHaveText('Not protected', {
      timeout: 15_000,
    });

    await mainPage.mixnetMode.click();
    await expect(mainPage.mixnetMode).toHaveAttribute('aria-pressed', 'true');
    await mainPage.connectButton.click();
    await expect(mainPage.disconnectButton).toBeVisible({ timeout: 15_000 });
    await mainPage.disconnectButton.click();
    await expect(mainPage.statusText).toHaveText('Not protected', {
      timeout: 15_000,
    });
  });

  test('keeps the mode toggle available while connected', async () => {
    await mainPage.connectButton.click();
    await expect(mainPage.disconnectButton).toBeVisible({ timeout: 15_000 });

    await expect(mainPage.fastMode).toBeVisible();
    await expect(mainPage.mixnetMode).toBeVisible();
  });
});
