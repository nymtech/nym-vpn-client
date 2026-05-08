import { Locator, Page } from '@playwright/test';

type Selectors = Record<string, Locator>;

class SettingsPage {
  readonly SELECTORS: Selectors;

  constructor(private readonly page: Page) {
    this.page = page;
    this.SELECTORS = {
      title: this.page.getByTestId('top-bar-title-text'),
      backButton: this.page
        .getByTestId('top-bar-left-button-container')
        .getByTestId('button-icon'),

      notification: this.page.getByTestId('notifications-toast'),

      // Settings rows
      accountButton: this.page.getByRole('button', { name: /Account/ }),

      supportFeedbackButton: this.page.getByRole('button', {
        name: /Support &/,
      }),

      killswitchButton: this.page.getByRole('button', { name: /Killswitch/ }),
      blockAdsButton: this.page.getByRole('button', { name: /Block ads/ }),
      supportIPv6Button: this.page.getByRole('button', {
        name: /Support IPv6/,
      }),
      bypassLANButton: this.page.getByRole('button', { name: /Bypass LAN/ }),
      customizeDNSButton: this.page.getByRole('button', {
        name: /Customize DNS/,
      }),
      antiCensorshipButton: this.page.getByRole('button', {
        name: /Anti-censorship/,
      }),
      appWalletProxyButton: this.page.getByRole('button', {
        name: /App & wallet proxy/,
      }),

      launchOnStartupButton: this.page.getByRole('button', {
        name: /Launch on/,
      }),
      appearanceButton: this.page.getByRole('button', { name: /Appearance/ }),
      desktopNotificationsButton: this.page.getByRole('button', {
        name: /Desktop notifications/,
      }),

      dataPrivacyButton: this.page.getByRole('button', {
        name: /Data, privacy/,
      }),

      legalButton: this.page.getByRole('button', { name: /Legal/ }),

      quitButton: this.page.getByRole('button', { name: 'Quit NymVPN' }),

      // App version
      clientVersionLabel: this.page.getByTestId('client-version-label'),
      clientVersionValue: this.page.getByTestId('client-version-value'),

      daemonVersionLabel: this.page.getByTestId('daemon-version-label'),
      daemonVersionValue: this.page.getByTestId('daemon-version-value'),

      networkNameLabel: this.page.getByTestId('network-name-label'),
      networkNameValue: this.page.getByTestId('network-name-value'),

      // Anti censorship
      enhancedConnectionText: this.page.getByRole('button', {
        name: 'Enhanced connection (QUIC)',
      }),
      howQUICImprovesLink: this.page.getByTestId(
        'link-how-quic-improves-connections',
      ),

      minimalObfuscationText: this.page.getByRole('button', {
        name: 'Minimal obfuscation (AmneziaWG)',
      }),
      howAmneziaWGPreventsDPILink: this.page.getByTestId(
        'link-how-amneziawg-prevents-dpi',
      ),

      stealAPIConnectText: this.page.getByRole('button', {
        name: 'Stealth API connect',
      }),
      howStealthAPIConnectWorksLink: this.page.getByTestId(
        'link-how-the-stealth-api-connect-mode-works',
      ),

      // DNS customization
      viewDefaultDNSButton: this.page.getByRole('button', {
        name: 'View default DNS',
      }),
      dnsAddressInput: this.page.getByRole('textbox', { name: 'DNS address' }),
      addDNSButton: this.page.getByRole('button', { name: 'Add' }),
      saveChangesButton: this.page.getByRole('button', {
        name: 'Save changes',
      }),
      deleteDNSButton: this.page
        .getByTestId('page-animation')
        .getByTestId('button-icon'),
      switchDNSButton: this.page
        .getByRole('button', { name: /Use custom DNS servers/ })
        .getByTestId('switch'),
      learnMoreAboutDNSLink: this.page.getByTestId('link-learn-more-about-dns'),
      invalidDNSAddressError: this.page.getByText('Invalid DNS address format'),

      // Appearance
      displayModeButton: this.page.getByRole('button', {
        name: /Display mode/,
      }),
      themesLabel: this.page.getByTestId('theme-radio-group-label'),
      automaticThemeButton: this.page.getByTestId(
        'theme-radio-group-option-system',
      ),
      lightThemeButton: this.page.getByTestId('theme-radio-group-option-light'),
      darkThemeButton: this.page.getByTestId('theme-radio-group-option-dark'),
      zoomSectionTitle: this.page.getByTestId('zoom-section-title'),

      // Account
      manageSubscriptionButton: this.page.getByRole('button', {
        name: /Manage your subscription/,
      }),
      accountId: this.page.getByText('Account ID', { exact: true }),
      deviceId: this.page.getByText('Device ID'),
      logoutButton: this.page.getByTestId('button'),
    };
  }

  async goto() {
    await this.page.goto('/settings');
  }

  async waitForPageLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  async clickBackButton() {
    await this.SELECTORS.backButton.click();
  }

  async clickAntiCensorshipButton() {
    await this.SELECTORS.antiCensorshipButton.click();
  }

  async clickCustomizeDNSButton() {
    await this.SELECTORS.customizeDNSButton.click();
  }

  async clickViewDefaultDNSButton() {
    await this.SELECTORS.viewDefaultDNSButton.click();
  }

  async fillDNSAddressInput(address: string) {
    await this.SELECTORS.dnsAddressInput.fill(address);
  }

  async clickAddDNSButton() {
    await this.SELECTORS.addDNSButton.click();
  }

  async clickSaveChangesButton() {
    await this.SELECTORS.saveChangesButton.click();
  }

  async clickDeleteDNSButton() {
    await this.SELECTORS.deleteDNSButton.click();
  }

  async clickSwitchDNSButton() {
    await this.SELECTORS.switchDNSButton.click();
  }

  async clickAppearanceButton() {
    await this.SELECTORS.appearanceButton.click();
  }

  async clickDisplayModeButton() {
    await this.SELECTORS.displayModeButton.click();
  }

  async clickDarkThemeButton() {
    await this.SELECTORS.darkThemeButton.click();
  }

  async clickLightThemeButton() {
    await this.SELECTORS.lightThemeButton.click();
  }

  async clickAccountButton() {
    await this.SELECTORS.accountButton.click();
  }

  async clickLogoutButton() {
    await this.SELECTORS.logoutButton.click();
  }
}

export default SettingsPage;
