import { expect, test } from '@playwright/test';
import MainPage from '../pages/MainPage';
import SettingsPage from '../pages/SettingsPage';

test.describe('Theme', () => {
  let settingsPage: SettingsPage;

  test.beforeEach(async ({ page }) => {
    const mainPage = new MainPage(page);
    settingsPage = new SettingsPage(page);
    await mainPage.goto();
    await mainPage.settingsButton.click();
    await settingsPage.openRow('Appearance');
    await settingsPage.displayModeRow.click();
    await expect(page).toHaveURL(/\/settings\/appearance\/display/);
  });

  test('renders theme switch screen correctly', async ({ page }) => {
    await expect(settingsPage.automaticTheme).toBeVisible();
    await expect(settingsPage.lightTheme).toBeVisible();
    await expect(settingsPage.darkTheme).toBeVisible();
    await expect(page.getByText('Zoom level')).toBeVisible();
  });

  test('marks the active theme as checked', async () => {
    await expect(settingsPage.darkTheme).toBeChecked();
    await expect(settingsPage.lightTheme).not.toBeChecked();
  });

  test('switches theme correctly', async () => {
    await settingsPage.lightTheme.click();
    await expect(settingsPage.themeSetter).toHaveAttribute(
      'data-test-theme',
      'light',
    );
    await expect(settingsPage.lightTheme).toBeChecked();

    await settingsPage.darkTheme.click();
    await expect(settingsPage.themeSetter).toHaveAttribute(
      'data-test-theme',
      'dark',
    );
    await expect(settingsPage.darkTheme).toBeChecked();
  });
});
