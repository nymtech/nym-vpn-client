import { test, expect } from '@playwright/test';
import MainPage from '../pages/MainPage';
import SettingsPage from '../pages/SettingsPage';

test.describe('Settings', () => {
  let mainPage: MainPage;
  let settingsPage: SettingsPage;

  test.beforeEach(async ({ page }) => {
    mainPage = new MainPage(page);
    settingsPage = new SettingsPage(page);
    await mainPage.goto();
    await mainPage.waitForPageLoad();
    await mainPage.clickSettingsButton();
  });

  test('renders settings screen correctly', async () => {
    await expect(settingsPage.SELECTORS.title).toBeVisible();
  });

  test('closes settings screen correctly', async () => {
    await expect(settingsPage.SELECTORS.title).toBeVisible();
    await settingsPage.clickBackButton();
    await mainPage.waitForPageLoad();
    await expect(mainPage.SELECTORS.connectionStatusText).toBeVisible();
    await expect(settingsPage.SELECTORS.title).not.toBeVisible();
  });

  test('renders all settings correctly', async () => {
    await expect(settingsPage.SELECTORS.accountButton).toBeVisible();
    await expect(settingsPage.SELECTORS.supportFeedbackButton).toBeVisible();
    await expect(settingsPage.SELECTORS.killswitchButton).toBeVisible();
    await expect(settingsPage.SELECTORS.blockAdsButton).toBeVisible();
    await expect(settingsPage.SELECTORS.supportIPv6Button).toBeVisible();
    await expect(settingsPage.SELECTORS.bypassLANButton).toBeVisible();
    await expect(settingsPage.SELECTORS.customizeDNSButton).toBeVisible();
    await expect(settingsPage.SELECTORS.antiCensorshipButton).toBeVisible();
    await expect(settingsPage.SELECTORS.appWalletProxyButton).toBeVisible();
    await expect(settingsPage.SELECTORS.launchOnStartupButton).toBeVisible();
    await expect(settingsPage.SELECTORS.appearanceButton).toBeVisible();
    await expect(
      settingsPage.SELECTORS.desktopNotificationsButton,
    ).toBeVisible();
    await expect(settingsPage.SELECTORS.dataPrivacyButton).toBeVisible();
    await expect(settingsPage.SELECTORS.legalButton).toBeVisible();
    await expect(settingsPage.SELECTORS.quitButton).toBeVisible();
    await expect(settingsPage.SELECTORS.clientVersionLabel).toBeVisible();
    await expect(settingsPage.SELECTORS.clientVersionValue).toBeVisible();
    await expect(settingsPage.SELECTORS.daemonVersionLabel).toBeVisible();
    await expect(settingsPage.SELECTORS.daemonVersionValue).toBeVisible();
    await expect(settingsPage.SELECTORS.networkNameLabel).toBeVisible();
    await expect(settingsPage.SELECTORS.networkNameValue).toBeVisible();
  });

  test('renders anti censorship settings correctly', async () => {
    await settingsPage.clickAntiCensorshipButton();

    await expect(settingsPage.SELECTORS.enhancedConnectionText).toBeVisible();
    await expect(settingsPage.SELECTORS.howQUICImprovesLink).toBeVisible();
    await expect(settingsPage.SELECTORS.minimalObfuscationText).toBeVisible();
    await expect(
      settingsPage.SELECTORS.howAmneziaWGPreventsDPILink,
    ).toBeVisible();
    await expect(settingsPage.SELECTORS.stealAPIConnectText).toBeVisible();
    await expect(
      settingsPage.SELECTORS.howStealthAPIConnectWorksLink,
    ).toBeVisible();
  });
});
