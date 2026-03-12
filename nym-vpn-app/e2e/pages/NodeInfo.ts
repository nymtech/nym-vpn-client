import { Locator, Page } from '@playwright/test';

type Selectors = Record<string, Locator>;

class NodeInfoPage {
  readonly SELECTORS: Selectors;

  constructor(private readonly page: Page) {
    this.page = page;
    this.SELECTORS = {
      title: this.page.getByTestId('top-bar-title-text'),

      eixtIPv4Text: this.page.getByText('Exit IPv4'),
      ASNText: this.page.getByText('ASN', { exact: true }),
      ASNNameText: this.page.getByText('ASN name'),
      advancedPrivacyText: this.page.getByText('Advanced privacy'),
      advancedPrivacyValue: this.page.getByText('With mixnet (5-hop)'),
      streamingContentText: this.page.getByText('Streaming & content'),
      streamingContentValue: this.page.getByText('Datacenter IP'),
      antiCensorshipText: this.page.getByText('Anti-censorship', {
        exact: true,
      }),
      antiCensorshipValue: this.page.getByText('QUIC protocol', {
        exact: true,
      }),

      enableQUICProtocolText: this.page.getByText('Enable “QUIC protocol” in'),

      overallPerformanceText: this.page.getByText('Overall performance'),
      overallPerformanceValue: this.page.getByText('Good'),
      serverLoadText: this.page.getByText('Server load', { exact: true }),
      serverLoadValue: this.page.getByText('Low'),
      uptimeText: this.page.getByText('Uptime', { exact: true }),
      uptimeValue: this.page.getByText('%'),

      performanceCalculatedText: this.page.getByText(
        'Performance score calculated',
      ),

      nymBuildVersionText: this.page.getByText('Nym build version'),
      identityKeyText: this.page.getByText('Identity key:'),
      copyIdentityKeyButton: this.page
        .getByTestId('page-animation')
        .getByTestId('button-icon'),

      incorrectInfoLink: this.page.getByTestId(
        'link-why-is-there-missing-or-incorrect-info',
      ),
      moreDetailsLink: this.page.getByText('More details in the Nym'),

      selectServerButton: this.page.getByTestId('button'),
    };
  }

  async waitForPageLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  async clickSelectServerButton() {
    await this.SELECTORS.selectServerButton.click();
  }
}

export default NodeInfoPage;
