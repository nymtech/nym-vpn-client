import { Locator, Page } from '@playwright/test';

/**
 * Home screen ("/home").
 *
 * Selectors are role/text based rather than test-id based: the redesigned UI
 * exposes almost no test ids, but its accessibility tree is rich and stable.
 * The one exception is the connection status label, which needs a stable hook
 * because its text is exactly what we assert on.
 */
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

  /**
   * The server row is the button following the given "Entry/Exit server"
   * label. In the disconnected state it reads "Random"; once connected it
   * shows the resolved gateway name.
   */
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
