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
    await expect(settingsPage.row('Kill switch')).toBeVisible();
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

  test('search keeps matches under their section with their description', async ({
    page,
  }) => {
    await settingsPage.search('dns');
    await expect(settingsPage.row('Customize DNS')).toBeVisible();
    await expect(page.getByText('Connection', { exact: true })).toBeVisible();
    await expect(settingsPage.row('Kill switch')).toBeHidden();

    await settingsPage.search('kill');
    await expect(settingsPage.row('Kill switch')).toContainText(
      'Always on to protect your data',
    );
    await expect(settingsPage.quitButton).toBeVisible();
  });

  test('search with no match shows no results and clearing restores the list', async () => {
    await settingsPage.search('zzz');
    await expect(settingsPage.noResults).toBeVisible();
    await expect(settingsPage.quitButton).toBeVisible();

    await settingsPage.search('');
    await expect(settingsPage.noResults).toBeHidden();
    await expect(settingsPage.row('Account')).toBeVisible();
    await expect(settingsPage.row('Legal')).toBeVisible();
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
    await expect(settingsPage.bridgeModeGroup).toBeVisible();
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
    await expect(settingsPage.bridgeModeGroup).toBeVisible();
    await expect(settingsPage.stealthApiToggle).toBeVisible();
  });

  test('selects the bridge mode', async () => {
    const off = settingsPage.bridgeModeOption('off');
    const auto = settingsPage.bridgeModeOption('auto');
    await expect(off).toHaveAttribute('aria-checked', 'true');
    await expect(auto).toHaveAttribute('aria-checked', 'false');

    await auto.click();

    await expect(auto).toHaveAttribute('aria-checked', 'true');
    await expect(off).toHaveAttribute('aria-checked', 'false');
  });
});
