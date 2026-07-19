import { expect, test } from '@playwright/test';

/**
 * Welcome screen ("/welcome").
 *
 * Replaces the former "/login" route, which no longer exists: account entry is
 * now driven from here rather than an in-app mnemonic form.
 */
test.describe('Welcome screen', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/welcome');
    await page.waitForLoadState('networkidle');
  });

  test('renders the welcome copy', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Welcome!' })).toBeVisible();
    await expect(
      page.getByText('To the world’s most private VPN.'),
    ).toBeVisible();
  });

  test('offers sign up and login actions', async ({ page }) => {
    await expect(page.getByRole('button', { name: 'Sign up' })).toBeVisible();
    await expect(
      page.getByRole('button', { name: 'Login to my account' }),
    ).toBeVisible();
  });

  test('shows the terms and privacy notice', async ({ page }) => {
    await expect(page.getByText(/By continuing, you agree/)).toBeVisible();
  });

  test('can be dismissed with the close button', async ({ page }) => {
    await page.getByRole('button', { name: 'close' }).click();

    await expect(page).not.toHaveURL(/\/welcome/);
  });
});
