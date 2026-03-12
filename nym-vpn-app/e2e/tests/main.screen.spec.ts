import { test, expect } from '@playwright/test';
import MainPage from '../pages/MainPage';
import SettingsPage from '../pages/SettingsPage';
import NodeListPage from '../pages/NodeList';

test.describe('MainScreen', () => {
  let mainPage: MainPage;

  test.beforeEach(async ({ page }) => {
    mainPage = new MainPage(page);
    await mainPage.goto();
    await mainPage.waitForPageLoad();
  });

  test('renders home screen correctly', async () => {
    const connectionStatusText = await mainPage.getConnectionStatusText();

    expect(connectionStatusText).toBe('Disconnected');
    await expect(mainPage.SELECTORS.connectionStatusText).toBeVisible();
    await expect(mainPage.SELECTORS.wireguardModeCard).toBeVisible();
    await expect(mainPage.SELECTORS.mixnetModeCard).toBeVisible();
    await expect(mainPage.SELECTORS.entryServer).toBeVisible();
    await expect(mainPage.SELECTORS.exitServer).toBeVisible();
    await expect(mainPage.SELECTORS.connectButton).toBeVisible();
    await expect(mainPage.SELECTORS.settingsButton).toBeVisible();
    await expect(mainPage.SELECTORS.modeInfoButton).toBeVisible();
  });

  test('navigates to settings screen correctly', async ({ page }) => {
    await mainPage.clickSettingsButton();

    await page.goto('/settings');
    await page.waitForLoadState('networkidle');

    const settingsPage = new SettingsPage(page);
    await settingsPage.waitForPageLoad();
    await expect(settingsPage.SELECTORS.title).toBeVisible();

    await settingsPage.clickBackButton();
    await mainPage.waitForPageLoad();
    await expect(mainPage.SELECTORS.connectionStatusText).toBeVisible();
    await expect(settingsPage.SELECTORS.title).not.toBeVisible();
  });

  test('switches to different modes correctly', async () => {
    await mainPage.clickWireguardModeCard();
    expect(await mainPage.isWireguardModeChecked()).toBe(true);
    expect(await mainPage.isMixnetModeChecked()).toBe(false);
    await mainPage.clickMixnetModeCard();
    expect(await mainPage.isMixnetModeChecked()).toBe(true);
    expect(await mainPage.isWireguardModeChecked()).toBe(false);
  });

  test('opens mode info dialog correctly', async () => {
    await mainPage.clickModeInfoButton();

    await expect(mainPage.SELECTORS.dialog).toBeVisible();
    await expect(mainPage.SELECTORS.title).toBeVisible();
    await expect(mainPage.SELECTORS.fastTitle).toBeVisible();
    await expect(mainPage.SELECTORS.mixnetTitle).toBeVisible();
    await expect(mainPage.SELECTORS.readMoreLink).toBeVisible();
    await expect(mainPage.SELECTORS.closeButton).toBeVisible();

    await mainPage.closeModeInfoDialog();

    await expect(mainPage.SELECTORS.dialog).not.toBeVisible();
  });

  test('switches to different entry and exit servers correctly', async ({
    page,
  }) => {
    await mainPage.clickEntryServer();

    const nodeListPage = new NodeListPage(page);
    await nodeListPage.waitForPageLoad();

    await nodeListPage.pickCountry('RU');
    await expect(page.getByText('Russia')).toBeVisible();
  });
});
