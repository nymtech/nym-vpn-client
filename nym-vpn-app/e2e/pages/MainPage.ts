import { Locator, Page } from '@playwright/test';

type Selectors = Record<string, Locator>;

class MainPage {
  readonly SELECTORS: Selectors;

  constructor(private readonly page: Page) {
    this.page = page;
    this.SELECTORS = {
      connectionStatusText: this.page.getByTestId('connection-status-text'),
      wireguardModeCard: this.page.getByTestId(
        'network-mode-radio-group-option-wg',
      ),
      mixnetModeCard: this.page.getByTestId(
        'network-mode-radio-group-option-mixnet',
      ),
      entryServer: this.page
        .locator('[data-testid="home-hop-selects-container"] >> text=Entry')
        .first()
        .locator('..'),
      exitServer: this.page
        .locator('[data-testid="home-hop-selects-container"] >> text=Exit')
        .locator('..'),
      connectButton: this.page.getByTestId('home-connection-button'),
      settingsButton: this.page
        .getByTestId('top-bar-right-button-container')
        .getByTestId('button-icon'),
      modeInfoButton: this.page
        .getByTestId('network-mode-label-container')
        .getByTestId('button-icon'),
      checkedMode: this.page.locator(
        '[data-testid="network-mode-radio-group-options-container"] [role="radio"][aria-checked="true"]',
      ),
      timer: this.page.getByTestId('connection-time-value'),
      timerLabel: this.page.getByTestId('connection-time-label'),

      // Mode Info Dialog
      dialog: this.page.getByTestId('mode-details-dialog-panel'),
      title: this.page.getByTestId('mode-details-title'),
      fastTitle: this.page.getByTestId('mode-details-fast-title'),
      mixnetTitle: this.page.getByTestId('mode-details-privacy-title'),
      readMoreLink: this.page.getByTestId('mode-details-learn-more-link'),
      closeButton: this.page.getByTestId('mode-details-close-button'),

      // Onboarding
      welcomeNYMVPNText: this.page.getByRole('heading', {
        name: 'Welcome to NymVPN',
      }),
      createAccountButton: this.page.getByRole('button', {
        name: 'Create account',
      }),
      loginButton: this.page.getByRole('button', { name: 'Log in' }),

      // Login page
      loginTitle: this.page.getByRole('heading', { name: 'Log in' }),
      mnemonicPhraseInput: this.page.getByTestId('login-mnemonic-input'),
      submitLoginButton: this.page.getByTestId('login-submit-button'),
      creatAccountLink: this.page.getByTestId('link-create-account'),
    };
  }

  async goto() {
    await this.page.goto('/');
  }

  async waitForPageLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  async getConnectionStatusText() {
    return await this.SELECTORS.connectionStatusText.textContent();
  }

  async clickConnectionButton() {
    await this.SELECTORS.connectButton.click();
  }

  async waitForConnectionStatusText(status: string) {
    await this.SELECTORS.connectionStatusText
      .getByText(status)
      .waitFor({ state: 'visible' });
  }

  async waitForConnected() {
    await this.waitForConnectionStatusText('Connected');
  }

  async waitForDisconnected() {
    await this.waitForConnectionStatusText('Disconnected');
  }

  async clickSettingsButton() {
    await this.SELECTORS.settingsButton.click({ force: true });
  }

  async clickModeInfoButton() {
    await this.SELECTORS.modeInfoButton.click();
  }

  async clickWireguardModeCard() {
    await this.SELECTORS.wireguardModeCard.click();
  }

  async clickMixnetModeCard() {
    await this.SELECTORS.mixnetModeCard.click();
  }

  async clickEntryServer() {
    await this.SELECTORS.entryServer.click();
  }

  async clickExitServer() {
    await this.SELECTORS.exitServer.click();
  }

  async isWireguardModeChecked() {
    return (await this.SELECTORS.checkedMode.getAttribute('data-key')) === 'wg';
  }

  async isMixnetModeChecked() {
    return (
      (await this.SELECTORS.checkedMode.getAttribute('data-key')) === 'mixnet'
    );
  }

  async closeModeInfoDialog() {
    await this.SELECTORS.closeButton.click();
  }

  async clickCreateAccountButton() {
    await this.SELECTORS.createAccountButton.click();
  }

  async clickLoginButton() {
    await this.SELECTORS.loginButton.click();
  }

  async fillMnemonicPhraseInput(mnemonic: string) {
    await this.SELECTORS.mnemonicPhraseInput.fill(mnemonic);
  }

  async clickSubmitLoginButton() {
    await this.SELECTORS.submitLoginButton.click();
  }

  async clickCreatAccountLink() {
    await this.SELECTORS.creatAccountLink.click();
  }
}

export default MainPage;
