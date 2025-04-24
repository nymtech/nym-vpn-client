const BasePage = require('./BasePage');

class Location extends BasePage {
  get dialogTitle() {
    return $('[data-testid="location-dialog-title"]');
  }
  get locationList() {
    return $('[data-testid="location-list"]');
  }
  get locationItems() {
    return $$('[data-testid="location-item"]');
  }
  get searchInput() {
    return $('[data-testid="location-search-input"]');
  }
  get closeButton() {
    return $('[data-testid="location-dialog-close"]');
  }

  async waitForPageLoad() {
    await super.waitForPageLoad();
    await this.waitForInteractable(this.dialogTitle);
    await this.waitForInteractable(this.locationList);
  }

  async selectLocation(countryName) {
    const countrySelector = `[data-testid="location-item"][data-country="${countryName}"]`;
    const countryElement = await $(countrySelector);

    await countryElement.scrollIntoView();

    let attempts = 0;
    const maxAttempts = 3;

    while (attempts < maxAttempts) {
      try {
        await this.clickElement(countryElement);
        await this.waitForAppUpdate();
        return;
      } catch (error) {
        attempts++;
        console.warn(
          `Attempt ${attempts} to click ${countryName} failed: ${error.message}`,
        );

        if (attempts >= maxAttempts) {
          throw new Error(
            `Failed to select location ${countryName} after ${maxAttempts} attempts`,
          );
        }

        await browser.pause(500);
      }
    }
  }

  async searchForLocation(searchTerm) {
    await this.setInputValue(this.searchInput, searchTerm);
    await this.waitForAppUpdate(1000);
  }

  async close() {
    await this.clickElement(this.closeButton);
    await this.waitForElementToDisappear(this.dialogTitle);
  }

  async getAvailableCountries() {
    await this.waitForInteractable(this.locationList);

    const locationItems = await this.locationItems;

    const countries = [];
    for (const item of locationItems) {
      const countryName = await item.getAttribute('data-country');
      countries.push(countryName);
    }

    return countries;
  }

  async isCountryAvailable(countryName) {
    const countries = await this.getAvailableCountries();
    return countries.includes(countryName);
  }

  async getCountryCount() {
    const countries = await this.getAvailableCountries();
    return countries.length;
  }
}

module.exports = new Location();
