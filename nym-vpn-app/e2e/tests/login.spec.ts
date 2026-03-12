import { test, expect } from '@playwright/test';
import MainPage from '../pages/MainPage';
import SettingsPage from '../pages/SettingsPage';

test.describe('Login', () => {
  let mainPage: MainPage;
  let settingsPage: SettingsPage;

  test.beforeEach(async ({ page }) => {
    mainPage = new MainPage(page);
    settingsPage = new SettingsPage(page);
    await mainPage.goto();
    await mainPage.waitForPageLoad();
    await mainPage.clickSettingsButton();
    await settingsPage.clickAccountButton();
  });

  test('renders my account screen correctly', async () => {
    await expect(settingsPage.SELECTORS.accountId).toBeVisible();
    await expect(settingsPage.SELECTORS.deviceId).toBeVisible();
    await expect(settingsPage.SELECTORS.manageSubscriptionButton).toBeVisible();
    await expect(settingsPage.SELECTORS.logoutButton).toBeVisible();
  });

  test('logs out correctly', async () => {
    await settingsPage.clickLogoutButton();
    await expect(settingsPage.SELECTORS.notification).toBeVisible();
  });

  test('logs in correctly', async ({ page }) => {
    await settingsPage.clickLogoutButton();
    await expect(settingsPage.SELECTORS.notification).toBeVisible();
    await settingsPage.clickBackButton();

    await settingsPage.clickBackButton();
    await expect(mainPage.SELECTORS.connectionStatusText).toBeVisible();

    await mainPage.clickConnectionButton();
    await expect(mainPage.SELECTORS.welcomeNYMVPNText).toBeVisible();
    await expect(mainPage.SELECTORS.createAccountButton).toBeVisible();
    await expect(mainPage.SELECTORS.loginButton).toBeVisible();

    await mainPage.clickLoginButton();

    await expect(mainPage.SELECTORS.loginTitle).toBeVisible();
    await expect(mainPage.SELECTORS.mnemonicPhraseInput).toBeVisible();
    await expect(mainPage.SELECTORS.submitLoginButton).toBeVisible();
    await expect(mainPage.SELECTORS.creatAccountLink).toBeVisible();

    await mainPage.fillMnemonicPhraseInput(
      'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about',
    );
    await mainPage.clickSubmitLoginButton();

    await mainPage.clickSettingsButton();
    await settingsPage.clickAccountButton();
    await expect(settingsPage.SELECTORS.accountId).toBeVisible();
    await expect(settingsPage.SELECTORS.deviceId).toBeVisible();
    await expect(settingsPage.SELECTORS.manageSubscriptionButton).toBeVisible();
    await expect(settingsPage.SELECTORS.logoutButton).toBeVisible();
  });
});
