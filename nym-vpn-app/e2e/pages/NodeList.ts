import { Locator, Page } from '@playwright/test';

/** Node location screen with Entry/Exit tabs. */
class NodeListPage {
  readonly backButton: Locator;
  readonly infoButton: Locator;
  readonly entryTab: Locator;
  readonly exitTab: Locator;
  readonly searchInput: Locator;
  readonly randomOption: Locator;
  readonly countSummary: Locator;
  readonly accordion: Locator;
  readonly expandButtons: Locator;
  readonly nodeDetailsButtons: Locator;
  readonly infoModal: Locator;
  readonly infoModalTitle: Locator;
  readonly streamingHeading: Locator;
  readonly locationAccuracyHeading: Locator;
  readonly closeModalButton: Locator;

  constructor(private readonly page: Page) {
    this.backButton = page.getByRole('button', { name: 'keyboard_arrow_left' });
    this.infoButton = page.getByRole('button', { name: 'info' }).first();
    this.entryTab = page.getByRole('tab', { name: 'Entry' });
    this.exitTab = page.getByRole('tab', { name: 'Exit' });
    this.searchInput = page.getByRole('textbox', { name: 'Search location' });
    this.randomOption = page.getByRole('button', { name: /Random/ }).first();
    this.countSummary = page.getByText(/\d+ countries · \d+ nodes/);
    this.accordion = page.getByTestId('node-list-accordion');
    this.expandButtons = page.getByRole('button', {
      name: 'keyboard_arrow_down',
    });
    this.nodeDetailsButtons = page.getByRole('button', {
      name: 'chevron_right',
    });

    this.infoModal = page.getByTestId('location-details-dialog-panel');
    this.infoModalTitle = page.getByTestId('location-details-title');
    this.streamingHeading = page.getByRole('heading', { name: 'Streaming' });
    this.locationAccuracyHeading = page.getByRole('heading', {
      name: 'Location accuracy',
    });
    this.closeModalButton = page.getByRole('button', { name: 'Ok' });
  }

  async goto() {
    await this.page.goto('/node-location');
    await this.waitForPageLoad();
  }

  async waitForPageLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  country(code: string): Locator {
    return this.page.getByTestId(`country-name-${code}`);
  }

  countryFlag(code: string): Locator {
    return this.page.getByTestId(`country-flag-${code}`);
  }

  async pickCountry(code: string) {
    await this.page.getByTestId(`country-info-${code}`).click();
  }

  /** Type key-by-key: the list's incremental filter ignores fill(). */
  async fillSearchInput(term: string) {
    await this.searchInput.click();
    await this.searchInput.fill('');
    await this.searchInput.pressSequentially(term, { delay: 30 });
  }

  async clearSearchInput() {
    await this.searchInput.click();
    await this.searchInput.fill('');
  }

  async getSearchInputValue() {
    return await this.searchInput.inputValue();
  }

  async clickExpandButton(index: number) {
    await this.expandButtons.nth(index).click();
  }

  async clickNodeDetailsButton(index: number) {
    await this.nodeDetailsButtons.nth(index).click();
  }

  async openInfoModal() {
    await this.infoButton.click();
  }

  async closeInfoModal() {
    await this.closeModalButton.click();
  }
}

export default NodeListPage;
