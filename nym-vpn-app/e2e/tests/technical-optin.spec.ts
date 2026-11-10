import { expect, test } from '@playwright/test';

const cases = [
  {
    label: 'Anonymous network stats',
    helpText: "Learn exactly what's shared",
    checked: 'true',
    expectedCommands: ['disable_netstats'],
  },
  {
    label: 'Error & crash reports',
    helpText: 'View Sentry Privacy Policy',
    checked: 'false',
    expectedCommands: ['enable_sentry', 'enable_netstats'],
  },
];

for (const { label, helpText, checked, expectedCommands } of cases) {
  test(`applies the telemetry preference for ${label}`, async ({ page }) => {
    const commands: string[] = [];
    page.on('console', (message) => {
      const match = message
        .text()
        .match(/^IPC call mocked "((?:enable|disable)_(?:sentry|netstats))"$/);
      if (match) commands.push(match[1]);
    });

    await page.goto('/technical-optin');
    await expect(page.getByText(helpText, { exact: true })).toBeVisible();
    const toggle = page.getByRole('switch', { name: label });
    await expect(toggle).toHaveAttribute('aria-checked', checked);
    await toggle.click();
    await page.getByRole('button', { name: 'Continue' }).click();

    await expect(page).toHaveURL(/\/home$/);
    expect(commands).toEqual(expectedCommands);
  });
}

test('exposes independent telemetry switches to keyboard users', async ({
  page,
}) => {
  await page.goto('/technical-optin');

  const networkStats = page.getByRole('switch', {
    name: 'Anonymous network stats',
  });
  const crashReports = page.getByRole('switch', {
    name: 'Error & crash reports',
  });

  await expect(page.getByRole('radio')).toHaveCount(0, { timeout: 1500 });
  await expect(networkStats).toBeChecked();
  await expect(crashReports).not.toBeChecked();

  await networkStats.focus();
  await page.keyboard.press('Space');
  await expect(networkStats).not.toBeChecked();
  await expect(crashReports).not.toBeChecked();

  await page.keyboard.press('Tab');
  await expect(crashReports).toBeFocused();
  await page.keyboard.press('Space');
  await expect(crashReports).toBeChecked();
  await expect(networkStats).not.toBeChecked();
});
