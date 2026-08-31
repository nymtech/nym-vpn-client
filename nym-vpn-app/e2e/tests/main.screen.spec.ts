import { expect, test } from '@playwright/test';
import MainPage from '../pages/MainPage';

test.describe('MainScreen', () => {
  let mainPage: MainPage;

  test.beforeEach(async ({ page }) => {
    mainPage = new MainPage(page);
    await mainPage.goto();
  });

  test('renders home screen correctly', async () => {
    await expect(mainPage.statusText).toHaveText('Not protected');
    await expect(mainPage.connectButton).toBeVisible();
    await expect(mainPage.settingsButton).toBeVisible();
    await expect(mainPage.fastMode).toBeVisible();
    await expect(mainPage.mixnetMode).toBeVisible();
    await expect(mainPage.entryServerLabel).toBeVisible();
    await expect(mainPage.exitServerLabel).toBeVisible();
  });

  test('selects Fast mode by default', async () => {
    expect(await mainPage.selectedMode()).toBe('Fast');
  });

  test('switches to different modes correctly', async () => {
    await mainPage.mixnetMode.click();
    await expect(mainPage.mixnetMode).toHaveAttribute('aria-pressed', 'true');
    await expect(mainPage.fastMode).toHaveAttribute('aria-pressed', 'false');

    await mainPage.fastMode.click();
    await expect(mainPage.fastMode).toHaveAttribute('aria-pressed', 'true');
    await expect(mainPage.mixnetMode).toHaveAttribute('aria-pressed', 'false');
  });

  test('renders entry and exit server rows', async () => {
    await expect(mainPage.serverRow('Entry')).toBeVisible();
    await expect(mainPage.serverRow('Exit')).toBeVisible();
    await expect(mainPage.serverRow('Entry')).toContainText(
      'Safest server selection',
    );
  });

  test('navigates to settings screen correctly', async ({ page }) => {
    await mainPage.settingsButton.click();

    await expect(page).toHaveURL(/\/settings$/);
    await expect(page.getByText('Settings', { exact: true })).toBeVisible();
  });

  test('closes settings screen correctly', async ({ page }) => {
    await mainPage.settingsButton.click();
    await expect(page).toHaveURL(/\/settings$/);

    await page.getByRole('button', { name: 'keyboard_arrow_left' }).click();

    await expect(page).toHaveURL(/\/home/);
    await expect(mainPage.statusText).toBeVisible();
  });

  test('opens node location from a server row', async ({ page }) => {
    await mainPage.serverRow('Entry').click();

    await expect(page).toHaveURL(/\/node-location/);
  });
});
