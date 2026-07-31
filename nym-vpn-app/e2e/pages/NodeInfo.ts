import { Locator, Page } from '@playwright/test';

/** Server details screen ("/node-details"), reached from a node row. */
class NodeInfoPage {
  readonly title: Locator;
  readonly backButton: Locator;
  readonly advancedPrivacyLabel: Locator;
  readonly advancedPrivacyValue: Locator;
  readonly streamingLabel: Locator;
  readonly streamingValue: Locator;
  readonly antiCensorshipLabel: Locator;
  readonly antiCensorshipValue: Locator;
  readonly enableQuicHint: Locator;
  readonly overallPerformanceLabel: Locator;
  readonly overallPerformanceValue: Locator;
  readonly serverLoadLabel: Locator;
  readonly serverLoadValue: Locator;
  readonly uptimeLabel: Locator;
  readonly uptimeValue: Locator;
  readonly performanceCalculatedNote: Locator;
  readonly exitIpv4Label: Locator;
  readonly asnLabel: Locator;
  readonly asnValue: Locator;
  readonly asnNameLabel: Locator;
  readonly asnNameValue: Locator;
  readonly buildVersionLabel: Locator;
  readonly identityKeyLabel: Locator;
  readonly copyIdentityKeyButton: Locator;
  readonly incorrectInfoLink: Locator;

  readonly selectServerButton: Locator;

  constructor(private readonly page: Page) {
    this.title = page.getByText('Server details', { exact: true });
    this.backButton = page.getByRole('button', { name: 'keyboard_arrow_left' });

    this.advancedPrivacyLabel = page.getByText('Advanced privacy');
    this.advancedPrivacyValue = page.getByText('Mixnet', { exact: true });
    this.streamingLabel = page.getByText('Streaming & IP');
    this.streamingValue = page.getByText('Datacenter', { exact: true });
    this.antiCensorshipLabel = page.getByText('Anti-censorship', {
      exact: true,
    });
    this.antiCensorshipValue = page.getByText('QUIC protocol', { exact: true });
    this.enableQuicHint = page.getByText('Enable “QUIC protocol” in');

    this.overallPerformanceLabel = page.getByText('Overall performance');
    this.overallPerformanceValue = page.getByText('Good', { exact: true });
    this.serverLoadLabel = page.getByText('Server load', { exact: true });
    this.serverLoadValue = page.getByText('Low', { exact: true });
    this.uptimeLabel = page.getByText('Uptime', { exact: true });
    this.uptimeValue = page.getByText(/%$/);
    this.performanceCalculatedNote = page.getByText(
      'Performance score calculated',
    );

    this.exitIpv4Label = page.getByText('Exit IPv4');
    this.asnLabel = page.getByText('ASN', { exact: true });
    this.asnValue = page.getByText(/^AS\d+$/);
    this.asnNameLabel = page.getByText('ASN name');
    this.asnNameValue = page.getByText('Akamai Connected Cloud');

    this.buildVersionLabel = page.getByText('Nym build version');
    this.identityKeyLabel = page.getByText('Identity key:');
    this.copyIdentityKeyButton = page.getByRole('button', {
      name: 'content_copy',
    });
    this.incorrectInfoLink = page.getByTestId(
      'link-why-is-there-missing-or-incorrect-info',
    );

    this.selectServerButton = page.getByRole('button', {
      name: 'Select server',
    });
  }

  async waitForPageLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  async clickSelectServerButton() {
    await this.selectServerButton.click();
  }
}

export default NodeInfoPage;
