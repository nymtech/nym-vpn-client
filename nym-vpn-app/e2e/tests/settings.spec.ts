import { expect, test } from '@playwright/test';
import MainPage from '../pages/MainPage';
import SettingsPage from '../pages/SettingsPage';

test.describe('Settings', () => {
  let settingsPage: SettingsPage;

  test.beforeEach(async ({ page }) => {
    settingsPage = new SettingsPage(page);
    await settingsPage.goto();
  });

  test('renders settings screen correctly', async ({ page }) => {
    await expect(page.getByText('Settings', { exact: true })).toBeVisible();
  });

  test('renders all settings correctly', async () => {
    await expect(settingsPage.row('Account')).toBeVisible();
    await expect(settingsPage.row('Support &')).toBeVisible();
    await expect(settingsPage.row('Killswitch')).toBeVisible();
    await expect(settingsPage.row('Block ads')).toBeVisible();
    await expect(settingsPage.row('Support IPv6')).toBeVisible();
    await expect(settingsPage.row('Bypass LAN')).toBeVisible();
    await expect(settingsPage.row('Customize DNS')).toBeVisible();
    await expect(settingsPage.row('Anti-censorship')).toBeVisible();
    await expect(settingsPage.row('App & wallet proxy')).toBeVisible();
    await expect(settingsPage.row('Launch on')).toBeVisible();
    await expect(settingsPage.row('Appearance')).toBeVisible();
    await expect(settingsPage.row('Data, privacy')).toBeVisible();
    await expect(settingsPage.row('Legal')).toBeVisible();
    await expect(settingsPage.quitButton).toBeVisible();
    await expect(settingsPage.appVersion).toBeVisible();
    await expect(settingsPage.daemonVersion).toBeVisible();
    await expect(settingsPage.networkName).toHaveText('mainnet');
  });

  test('closes settings screen correctly', async ({ page }) => {
    const mainPage = new MainPage(page);
    await mainPage.goto();
    await mainPage.settingsButton.click();
    await expect(page).toHaveURL(/\/settings$/);

    await settingsPage.backButton.click();

    await expect(page).toHaveURL(/\/home/);
    await expect(mainPage.statusText).toBeVisible();
  });

  test('navigates to DNS customization', async ({ page }) => {
    await settingsPage.openRow('Customize DNS');

    await expect(page).toHaveURL(/\/settings\/dns/);
    await expect(settingsPage.dnsAddressInput).toBeVisible();
  });

  test('navigates to anti-censorship', async ({ page }) => {
    await settingsPage.openRow('Anti-censorship');

    await expect(page).toHaveURL(/\/settings\/anti-censorship/);
    await expect(settingsPage.quicToggle).toBeVisible();
  });

  test('navigates to appearance', async ({ page }) => {
    await settingsPage.openRow('Appearance');

    await expect(page).toHaveURL(/\/settings\/appearance/);
    await expect(settingsPage.displayModeRow).toBeVisible();
  });

  test('navigates to notifications', async ({ page }) => {
    await settingsPage.openRow('Notifications');

    await expect(page).toHaveURL(/\/settings\/notifications/);
  });
});

test.describe('Settings - anti censorship', () => {
  let settingsPage: SettingsPage;

  test.beforeEach(async ({ page }) => {
    settingsPage = new SettingsPage(page);
    await settingsPage.goto('/settings/anti-censorship');
  });

  test('renders anti censorship settings correctly', async () => {
    await expect(settingsPage.quicToggle).toBeVisible();
    await expect(settingsPage.amneziaToggle).toBeVisible();
    await expect(settingsPage.stealthApiToggle).toBeVisible();
  });

  test('toggles the QUIC option', async () => {
    const quicSwitch = settingsPage.quicToggle.getByRole('switch');
    await expect(quicSwitch).not.toBeChecked();

    await settingsPage.quicToggle.click();

    await expect(quicSwitch).toBeChecked();
  });

  test('keeps AmneziaWG locked on', async () => {
    const amneziaSwitch = settingsPage.amneziaToggle.getByRole('switch');

    await expect(amneziaSwitch).toBeChecked();
    await expect(amneziaSwitch).toBeDisabled();
  });
});
