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
    const isMacOS = process.platform === 'darwin';

    if (isMacOS) {
      // On macOS, we use browser testing via localhost
      await browser.url(
        'http://localhost:1420' + (path.startsWith('/') ? path : '/' + path),
      );
    } else {
      // For Linux/Windows, we're testing the native app directly
      // No need to navigate to a URL since we're already connected to the app window
      console.log(
        'Connected to native Tauri application, no URL navigation needed',
      );
    }

    await this.waitForPageLoad();
  }

  async waitForPageLoad() {
    // Add longer wait time for debug mode
    const isDebug = process.env.DEBUG === 'true' || process.env.DEBUG === true;
    const waitTime = isDebug ? 1000 : 500;
    await waitForTauriRerender(waitTime);
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
