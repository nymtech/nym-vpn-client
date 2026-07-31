import { expect, test } from '@playwright/test';
import SettingsPage from '../pages/SettingsPage';

const DNS_ADDRESS = '192.168.1.1';
const INVALID_DNS_ADDRESS = '192.168.1.1.1';

test.describe('DNS', () => {
  let settingsPage: SettingsPage;

  test.beforeEach(async ({ page }) => {
    settingsPage = new SettingsPage(page);
    await settingsPage.goto('/settings/dns');
  });

  test('renders DNS customization screen correctly', async () => {
    await expect(settingsPage.viewDefaultDnsButton).toBeVisible();
    await expect(settingsPage.dnsAddressInput).toBeVisible();
    await expect(settingsPage.addDnsButton).toBeVisible();
    await expect(settingsPage.saveChangesButton).toBeVisible();
    await expect(settingsPage.learnMoreAboutDnsLink).toBeVisible();
  });

  test('disables custom DNS and saving until a server is added', async () => {
    await expect(
      settingsPage.customDnsToggle.getByRole('switch'),
    ).toBeDisabled();
    await expect(settingsPage.saveChangesButton).toBeDisabled();
  });

  test('displays error message for invalid DNS address', async () => {
    await settingsPage.dnsAddressInput.fill(INVALID_DNS_ADDRESS);
    await settingsPage.addDnsButton.click();

    await expect(settingsPage.invalidDnsError).toBeVisible();
    await expect(settingsPage.saveChangesButton).toBeDisabled();
  });

  test('adds custom DNS correctly', async ({ page }) => {
    await settingsPage.dnsAddressInput.fill(DNS_ADDRESS);
    await settingsPage.addDnsButton.click();

    await expect(settingsPage.customDnsList).toBeVisible();
    await expect(page.getByText(DNS_ADDRESS)).toBeVisible();
    await expect(settingsPage.deleteDnsButton).toBeVisible();
    await expect(settingsPage.saveChangesButton).toBeEnabled();

    await settingsPage.saveChangesButton.click();

    await expect(settingsPage.toast).toBeVisible();
    await expect(
      settingsPage.customDnsToggle.getByRole('switch'),
    ).toBeEnabled();
    await expect(settingsPage.saveChangesButton).toBeDisabled();
  });

  test('deletes custom DNS correctly', async ({ page }) => {
    await settingsPage.dnsAddressInput.fill(DNS_ADDRESS);
    await settingsPage.addDnsButton.click();
    await settingsPage.saveChangesButton.click();
    await expect(page.getByText(DNS_ADDRESS)).toBeVisible();

    await settingsPage.deleteDnsButton.click();
    await settingsPage.saveChangesButton.click();

    await expect(page.getByText(DNS_ADDRESS)).toHaveCount(0);
    await expect(settingsPage.deleteDnsButton).toHaveCount(0);
  });
});
