import { Locator, Page } from '@playwright/test';

type Selectors = Record<string, Locator>;

class NodeListPage {
  readonly SELECTORS: Selectors;

  constructor(private readonly page: Page) {
    this.page = page;
    this.SELECTORS = {
      title: this.page.getByTestId('top-bar-title-text'),
      searchInput: this.page.getByRole('textbox', { name: 'Search' }),
      countryItems: this.page.getByTestId(/country-info/),
      infoButton: this.page
        .getByTestId('top-bar-right-button-container')
        .getByTestId('button-icon'),
      clearSearchButton: this.page
        .getByTestId('node-search-container')
        .getByTestId('button-icon'),
      backButton: this.page
        .getByTestId('top-bar-left-button-container')
        .getByTestId('button-icon'),
      expandButton: this.page.getByRole('button', {
        name: 'keyboard_arrow_down',
      }),
      nodeDetailsButton: this.page.getByRole('button', { name: 'arrow_right' }),

      // Info modal
      nodeInfoModal: this.page.getByTestId('location-details-dialog-panel'),
      closeModalButton: this.page.getByTestId('location-details-close-button'),
      quicProtocolText: this.page.getByRole('heading', {
        name: 'QUIC protocol',
      }),
      streamingText: this.page.getByRole('heading', { name: 'Streaming' }),
      locationAccuracyText: this.page.getByRole('heading', {
        name: 'Location accuracy',
      }),
    };
  }

  async gotoEntryServer() {
    await this.page.goto('/entry-node-location');
  }

  async gotoExitServer() {
    await this.page.goto('/exit-node-location');
  }

  async waitForPageLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  async pickCountry(country: string) {
    await this.page.getByTestId(`country-info-${country}`).click();
  }

  async clickInfoButton() {
    await this.SELECTORS.infoButton.click();
  }

  async clickClearSearchButton() {
    await this.SELECTORS.clearSearchButton.click();
  }

  async fillSearchInput(search: string) {
    await this.SELECTORS.searchInput.fill(search);
  }

  async getSearchInputValue() {
    return await this.SELECTORS.searchInput.inputValue();
  }

  async closeInfoModal() {
    await this.SELECTORS.closeModalButton.click();
  }

  async clickExpandButton(number: number) {
    await this.SELECTORS.expandButton.nth(number).click();
  }

  async clickNodeDetailsButton(number: number) {
    await this.SELECTORS.nodeDetailsButton.nth(number).click();
  }
}

export default NodeListPage;
