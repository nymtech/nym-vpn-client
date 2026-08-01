import { expect, test } from '@playwright/test';
import MainPage from '../pages/MainPage';
import SettingsPage from '../pages/SettingsPage';

const MNEMONIC =
  'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';

/** Account / login journey (no /login route; reached by logging out). */
test.describe('Login', () => {
  let mainPage: MainPage;
  let settingsPage: SettingsPage;

  test.beforeEach(async ({ page }) => {
    mainPage = new MainPage(page);
    settingsPage = new SettingsPage(page);
    await mainPage.goto();
    await mainPage.settingsButton.click();
    await settingsPage.accountRow.click();
    await expect(page).toHaveURL(/\/settings\/account/);
  });

  test('renders my account screen correctly', async ({ page }) => {
    await expect(settingsPage.accountId).toBeVisible();
    await expect(settingsPage.deviceId).toBeVisible();
    await expect(
      page.getByRole('button', { name: /Manage your subscription/ }),
    ).toBeVisible();
    await expect(settingsPage.logoutButton).toBeVisible();
  });

  test('logs out correctly', async ({ page }) => {
    await settingsPage.logoutButton.click();

    await expect(settingsPage.toast).toBeVisible();
    await expect(page).toHaveURL(/\/settings$/);
    await expect(settingsPage.getStartedButton).toBeVisible();
  });

  test('walks from logout to the passphrase form', async ({ page }) => {
    await settingsPage.logoutButton.click();
    await expect(settingsPage.getStartedButton).toBeVisible();

    await settingsPage.getStartedButton.click();
    await expect(page).toHaveURL(/\/hideout\/onboarding/);

    await page.getByRole('button', { name: 'Get Started' }).click();
    await expect(page).toHaveURL(/\/welcome/);
    await expect(page.getByRole('heading', { name: 'Welcome!' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Sign up' })).toBeVisible();

    await page.getByRole('button', { name: 'Login to my account' }).click();
    await expect(page.getByRole('heading', { name: 'Log in' })).toBeVisible();
    await expect(
      page.getByRole('button', { name: 'Login with 24 words' }),
    ).toBeVisible();

    await page.getByRole('button', { name: 'Login with 24 words' }).click();
    await expect(
      page.getByRole('textbox', { name: /24-word passphrase/ }),
    ).toBeVisible();
    await expect(page.getByRole('button', { name: 'Login' })).toBeVisible();
  });

  test('logs in correctly', async ({ page }) => {
    await settingsPage.logoutButton.click();
    await settingsPage.getStartedButton.click();
    await page.getByRole('button', { name: 'Get Started' }).click();
    await page.getByRole('button', { name: 'Login to my account' }).click();
    await page.getByRole('button', { name: 'Login with 24 words' }).click();

    await page
      .getByRole('textbox', { name: /24-word passphrase/ })
      .fill(MNEMONIC);
    await page.getByRole('button', { name: 'Login' }).click();
    await mainPage.settingsButton.click();
    await settingsPage.accountRow.click();
    await expect(settingsPage.accountId).toBeVisible();
    await expect(settingsPage.deviceId).toBeVisible();
    await expect(settingsPage.logoutButton).toBeVisible();
  });
});
