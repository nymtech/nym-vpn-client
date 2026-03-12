import { test, expect } from '@playwright/test';
import MainPage from '../pages/MainPage';
import SettingsPage from '../pages/SettingsPage';

test.describe('DNS', () => {
  let mainPage: MainPage;
  let settingsPage: SettingsPage;
  const DNS_ADDRESS = '192.168.1.1';
  const INVALID_DNS_ADDRESS = '192.168.1.1.1';

  test.beforeEach(async ({ page }) => {
    mainPage = new MainPage(page);
    settingsPage = new SettingsPage(page);
    await mainPage.goto();
    await mainPage.waitForPageLoad();
    await mainPage.clickSettingsButton();
    await settingsPage.clickCustomizeDNSButton();
  });

  test('renders DNS customization screen correctly', async () => {
    await expect(settingsPage.SELECTORS.viewDefaultDNSButton).toBeVisible();
    await expect(settingsPage.SELECTORS.dnsAddressInput).toBeVisible();
    await expect(settingsPage.SELECTORS.addDNSButton).toBeVisible();
    await expect(settingsPage.SELECTORS.saveChangesButton).toBeVisible();
    await expect(settingsPage.SELECTORS.learnMoreAboutDNSLink).toBeVisible();
  });

  test('adds custom DNS correctly', async ({ page }) => {
    await expect(settingsPage.SELECTORS.switchDNSButton).toBeDisabled();
    await settingsPage.fillDNSAddressInput(DNS_ADDRESS);
    await settingsPage.clickAddDNSButton();
    await settingsPage.clickSaveChangesButton();

    await expect(settingsPage.SELECTORS.notification).toBeVisible();
    await expect(page.getByText(DNS_ADDRESS)).toBeVisible();
    await expect(settingsPage.SELECTORS.deleteDNSButton).toBeVisible();
    await expect(settingsPage.SELECTORS.switchDNSButton).toBeEnabled();
  });

  test('deletes custom DNS correctly', async ({ page }) => {
    await settingsPage.fillDNSAddressInput(DNS_ADDRESS);
    await settingsPage.clickAddDNSButton();
    await settingsPage.clickSaveChangesButton();
    await expect(settingsPage.SELECTORS.notification).toBeVisible();
    await expect(page.getByText(DNS_ADDRESS)).toBeVisible();

    await settingsPage.clickDeleteDNSButton();
    await settingsPage.clickSaveChangesButton();
    await expect(settingsPage.SELECTORS.notification).toBeVisible();
    await expect(page.getByText(DNS_ADDRESS)).not.toBeVisible();
    await expect(settingsPage.SELECTORS.deleteDNSButton).not.toBeVisible();
    await expect(settingsPage.SELECTORS.switchDNSButton).toBeDisabled();
  });

  test('displays error message for invalid DNS address', async () => {
    await settingsPage.fillDNSAddressInput(INVALID_DNS_ADDRESS);
    await settingsPage.clickAddDNSButton();
    await expect(settingsPage.SELECTORS.invalidDNSAddressError).toBeVisible();
  });
});
