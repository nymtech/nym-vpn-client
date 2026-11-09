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
    const card = page.getByRole('radio').filter({ hasText: label });
    await expect(card.getByText(helpText, { exact: true })).toBeVisible();
    const toggle = card.getByRole('switch');
    await expect(toggle).toHaveAttribute('aria-checked', checked);
    await toggle.click();
    await page.getByRole('button', { name: 'Continue' }).click();

    await expect(page).toHaveURL(/\/home$/);
    expect(commands).toEqual(expectedCommands);
  });
}
