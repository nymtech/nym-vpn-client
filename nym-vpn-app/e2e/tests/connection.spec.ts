import { test, expect } from '@playwright/test';
import MainPage from '../pages/MainPage';

test.describe('Connection', () => {
  let mainPage: MainPage;

  test.beforeEach(async ({ page }) => {
    mainPage = new MainPage(page);
    await mainPage.goto();
    await mainPage.waitForPageLoad();
  });

  test('connects to VPN correctly', async () => {
    expect(await mainPage.getConnectionStatusText()).toBe('Disconnected');
    await mainPage.clickConnectionButton();

    expect(await mainPage.getConnectionStatusText()).toBe('Connecting');

    await mainPage.waitForConnected();
    expect(await mainPage.getConnectionStatusText()).toBe('Connected');
    await expect(mainPage.SELECTORS.timer).toBeVisible();
    await expect(mainPage.SELECTORS.timerLabel).toBeVisible();
  });

  test('disconnects from VPN correctly', async ({ page }) => {
    await mainPage.clickConnectionButton();
    await mainPage.waitForConnected();
    await mainPage.clickConnectionButton();
    await mainPage.waitForDisconnected();

    expect(await mainPage.getConnectionStatusText()).toBe('Disconnected');
    await expect(mainPage.SELECTORS.timer).not.toBeVisible();
    await expect(mainPage.SELECTORS.timerLabel).not.toBeVisible();
  });

  test('connects from different modes correctly', async ({ page }) => {
    await mainPage.clickWireguardModeCard();
    await mainPage.clickConnectionButton();
    await mainPage.waitForConnected();
    expect(await mainPage.getConnectionStatusText()).toBe('Connected');
    await mainPage.clickConnectionButton();
    await mainPage.waitForDisconnected();
    expect(await mainPage.getConnectionStatusText()).toBe('Disconnected');

    await mainPage.clickMixnetModeCard();
    await mainPage.clickConnectionButton();
    await mainPage.waitForConnected();
    expect(await mainPage.getConnectionStatusText()).toBe('Connected');
    await mainPage.clickConnectionButton();
    await mainPage.waitForDisconnected();
    expect(await mainPage.getConnectionStatusText()).toBe('Disconnected');
  });
});
