const {
  waitForInteractable,
  waitAndClick,
  waitAndGetText,
  waitAndSetValue,
  waitForTextToContain,
  waitForTextToEqual,
  waitForElementToDisappear,
  waitForAttributeToEqual,
  waitForTauriRerender,
} = require('../utils/test-utils');

class BasePage {
  async open(path = '') {
    // Modified to use the absolute URL for browser testing
    // This will work for both macOS and other platforms without platform detection
    await browser.url(
      'http://localhost:1420' + (path.startsWith('/') ? path : '/' + path),
    );
    await this.waitForPageLoad();
  }

  async waitForPageLoad() {
    await waitForTauriRerender();
  }

  async clickElement(element, timeout = 5000) {
    await waitAndClick(element, timeout);
  }

  async getElementText(element, timeout = 5000) {
    return await waitAndGetText(element, timeout);
  }

  async setInputValue(element, value, timeout = 5000) {
    await waitAndSetValue(element, value, timeout);
  }

  async waitForElementTextToContain(element, text, timeout = 5000) {
    return await waitForTextToContain(element, text, timeout);
  }

  async waitForElementTextToEqual(element, text, timeout = 5000) {
    return await waitForTextToEqual(element, text, timeout);
  }

  async waitForElementToDisappear(element, timeout = 5000) {
    return await waitForElementToDisappear(element, timeout);
  }

  async waitForElementAttributeToEqual(
    element,
    attribute,
    value,
    timeout = 5000,
  ) {
    return await waitForAttributeToEqual(element, attribute, value, timeout);
  }

  async waitForAppUpdate(ms = 500) {
    await waitForTauriRerender(ms);
  }
}

module.exports = BasePage;
