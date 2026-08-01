import { Locator, Page } from '@playwright/test';

/** Home screen. Role/text-based selectors (redesign exposes few test ids). */
class MainPage {
  readonly themeButton: Locator;
  readonly settingsButton: Locator;
  readonly statusText: Locator;
  readonly fastMode: Locator;
  readonly mixnetMode: Locator;
  readonly entryServerLabel: Locator;
  readonly exitServerLabel: Locator;
  readonly connectButton: Locator;
  readonly cancelButton: Locator;
  readonly disconnectButton: Locator;
  readonly getStartedButton: Locator;
  readonly connectionTimer: Locator;

  constructor(private readonly page: Page) {
    this.themeButton = page.getByTestId('top-bar-left-button-container');
    this.settingsButton = page.getByRole('button', { name: 'settings' });
    this.statusText = page.getByTestId('connection-status-text');
    this.fastMode = page.getByRole('button', { name: 'Fast', exact: true });
    this.mixnetMode = page.getByRole('button', { name: 'Mixnet', exact: true });
    this.entryServerLabel = page.getByText('Entry server', { exact: true });
    this.exitServerLabel = page.getByText('Exit server', { exact: true });
    this.connectButton = page.getByRole('button', { name: 'Tap to connect' });
    this.cancelButton = page.getByRole('button', { name: 'Tap to cancel' });
    this.disconnectButton = page.getByRole('button', {
      name: 'Tap to disconnect',
    });
    this.getStartedButton = page.getByRole('button', { name: 'Get started' });
    this.connectionTimer = page.getByTestId('connection-timer');
  }

  async goto() {
    await this.page.goto('/home');
    await this.waitForPageLoad();
  }

  async waitForPageLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  /** Server row: the button after the Entry/Exit server label. */
  serverRow(hop: 'Entry' | 'Exit'): Locator {
    return this.page.locator(
      `xpath=//p[normalize-space(text())="${hop} server"]` +
        `/following-sibling::div[@role="button"][1]`,
    );
  }

  async selectedMode(): Promise<'Fast' | 'Mixnet' | null> {
    if ((await this.fastMode.getAttribute('aria-pressed')) === 'true') {
      return 'Fast';
    }
    if ((await this.mixnetMode.getAttribute('aria-pressed')) === 'true') {
      return 'Mixnet';
    }
    return null;
  }
}

export default MainPage;
