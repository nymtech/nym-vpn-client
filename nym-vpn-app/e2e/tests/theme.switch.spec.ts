import { test, expect } from '@playwright/test';
import MainPage from '../pages/MainPage';
import SettingsPage from '../pages/SettingsPage';

test.describe('Theme', () => {
  let mainPage: MainPage;
  let settingsPage: SettingsPage;

  test.beforeEach(async ({ page }) => {
    mainPage = new MainPage(page);
    settingsPage = new SettingsPage(page);
    await mainPage.goto();
    await mainPage.waitForPageLoad();
    await mainPage.clickSettingsButton();
    await settingsPage.clickAppearanceButton();
    await settingsPage.clickDisplayModeButton();
  });

  test('renders theme switch screen correctly', async () => {
    await expect(settingsPage.SELECTORS.themesLabel).toBeVisible();
    await expect(settingsPage.SELECTORS.automaticThemeButton).toBeVisible();
    await expect(settingsPage.SELECTORS.lightThemeButton).toBeVisible();
    await expect(settingsPage.SELECTORS.darkThemeButton).toBeVisible();
    await expect(settingsPage.SELECTORS.zoomSectionTitle).toBeVisible();
  });

  test('switches theme correctly', async ({ page }) => {
    await settingsPage.clickDarkThemeButton();
    await expect(page.getByTestId('theme-setter')).toHaveAttribute(
      'data-test-theme',
      'dark',
    );

    await settingsPage.clickLightThemeButton();
    await expect(page.getByTestId('theme-setter')).toHaveAttribute(
      'data-test-theme',
      'light',
    );
  });
});
