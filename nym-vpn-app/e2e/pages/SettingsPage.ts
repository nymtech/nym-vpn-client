import { Locator, Page } from '@playwright/test';

/** Settings index and sub-screens (menu rows matched by label substring). */
class SettingsPage {
  readonly backButton: Locator;
  readonly quitButton: Locator;
  readonly appVersion: Locator;
  readonly daemonVersion: Locator;
  readonly networkName: Locator;
  readonly toast: Locator;
  readonly accountRow: Locator;
  readonly logoutButton: Locator;
  readonly accountId: Locator;
  readonly deviceId: Locator;
  readonly getStartedButton: Locator;
  readonly quicToggle: Locator;
  readonly amneziaToggle: Locator;
  readonly stealthApiToggle: Locator;
  readonly viewDefaultDnsButton: Locator;
  readonly customDnsToggle: Locator;
  readonly dnsAddressInput: Locator;
  readonly addDnsButton: Locator;
  readonly saveChangesButton: Locator;
  readonly learnMoreAboutDnsLink: Locator;
  readonly deleteDnsButton: Locator;
  readonly invalidDnsError: Locator;
  readonly customDnsList: Locator;
  readonly displayModeRow: Locator;
  readonly languageRow: Locator;
  readonly automaticTheme: Locator;
  readonly lightTheme: Locator;
  readonly darkTheme: Locator;
  /** Wrapper that carries the active theme via `data-test-theme`. */
  readonly themeSetter: Locator;

  constructor(private readonly page: Page) {
    this.backButton = page.getByRole('button', { name: 'keyboard_arrow_left' });
    this.quitButton = page.getByRole('button', { name: 'Quit NymVPN' });
    this.appVersion = page.getByTestId('client-version-value');
    this.daemonVersion = page.getByTestId('daemon-version-value');
    this.networkName = page.getByTestId('network-name-value');
    this.toast = page.getByRole('dialog');

    this.accountRow = page.getByRole('button', { name: /Account/ }).first();
    this.logoutButton = page.getByRole('button', { name: /Log ?out/i });
    this.accountId = page.getByText('Account ID', { exact: true });
    this.deviceId = page.getByText('Device ID', { exact: true });
    this.getStartedButton = page.getByRole('button', { name: 'Get started' });

    this.quicToggle = page.getByRole('button', {
      name: /Enhanced connection \(QUIC\)/,
    });
    this.amneziaToggle = page.getByRole('button', {
      name: /Minimal obfuscation \(AmneziaWG\)/,
    });
    this.stealthApiToggle = page.getByRole('button', {
      name: /Stealth API connect/,
    });

    this.viewDefaultDnsButton = page.getByRole('button', {
      name: 'View default DNS',
    });
    this.customDnsToggle = page.getByRole('button', {
      name: /Use custom DNS servers/,
    });
    this.dnsAddressInput = page.getByRole('textbox', {
      name: 'IPv4 or IPv6 address',
    });
    this.addDnsButton = page.getByRole('button', { name: 'Add', exact: true });
    this.saveChangesButton = page.getByRole('button', { name: 'Save changes' });
    this.learnMoreAboutDnsLink = page.getByTestId('link-learn-more-about-dns');
    this.deleteDnsButton = page.getByRole('button', { name: 'delete_outline' });
    this.invalidDnsError = page.getByText('Invalid DNS address format');
    this.customDnsList = page.getByText(/Custom DNS servers \(\d\/\d\)/);

    this.displayModeRow = page.getByRole('button', { name: /Display mode/ });
    this.languageRow = page.getByRole('button', { name: /Language/ });
    this.automaticTheme = page.getByRole('radio', { name: /Automatic/ });
    this.lightTheme = page.getByRole('radio', { name: /Light theme/ });
    this.darkTheme = page.getByRole('radio', { name: /Dark theme/ });
    this.themeSetter = page.getByTestId('theme-setter');
  }

  async goto(path = '/settings') {
    await this.page.goto(path);
    await this.waitForPageLoad();
  }

  async waitForPageLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  /** A settings menu row, matched on a substring of its label. */
  row(label: string | RegExp): Locator {
    return this.page
      .getByRole('button', {
        name: typeof label === 'string' ? new RegExp(label) : label,
      })
      .first();
  }

  async openRow(label: string | RegExp) {
    await this.row(label).click();
  }
}

export default SettingsPage;
