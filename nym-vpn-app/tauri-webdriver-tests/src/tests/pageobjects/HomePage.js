const BasePage = require('./BasePage');
const { isCI, getTimeout } = require('../utils/environment-config');

class HomePage extends BasePage {
  get themeSetter() {
    return $('[data-testid="theme-setter"]');
  }
  get topBar() {
    return $('[data-testid="top-bar"]');
  }
  get notificationsViewport() {
    return $('[data-testid="notifications-viewport"]');
  }
  get topBarRightButton() {
    return $('[data-testid="top-bar-right-button"]');
  }
  get topBarRightIcon() {
    return $('[data-testid="top-bar-right-icon"]');
  }

  // Connection status elements
  get homeContainer() {
    return $('[data-testid="home-container"]');
  }
  get tunnelStateContainer() {
    return $('[data-testid="tunnel-state-container"]');
  }
  get tunnelBadgeContainer() {
    return $('[data-testid="tunnel-badge-container"]');
  }
  get connectionBadge() {
    return $('[data-testid="connection-badge"]');
  }
  get connectionStatusText() {
    return $('[data-testid="connection-status-text"]');
  }
  get tunnelDetailsContainer() {
    return $('[data-testid="tunnel-details-container"]');
  }

  // Network mode selection elements
  get homeControlsContainer() {
    return $('[data-testid="home-controls-container"]');
  }
  get networkModeSelectContainer() {
    return $('[data-testid="network-mode-select-container"]');
  }
  get networkModeLabel() {
    return $('[data-testid="network-mode-label"]');
  }
  get networkModeInfoButton() {
    return $('[data-testid="network-mode-info-button"]');
  }
  get networkModeRadioGroupContainer() {
    return $('[data-testid="network-mode-radio-group-container"]');
  }
  get networkModeRadioGroup() {
    return $('[data-testid="network-mode-radio-group"]');
  }
  get networkModeRadioGroupOptionsContainer() {
    return $('[data-testid="network-mode-radio-group-options-container"]');
  }

  // Network mode options
  get networkModeOptionWireguard() {
    return $('[data-testid="network-mode-radio-group-option-wg"]');
  }
  get networkModeFastIcon() {
    return $('[data-testid="network-mode-fast-icon"]');
  }
  get networkModeOptionWireguardLabel() {
    return $('[data-testid="network-mode-radio-group-option-wg-label"]');
  }
  get networkModeOptionWireguardDescription() {
    return $('[data-testid="network-mode-radio-group-option-wg-description"]');
  }

  get networkModeOptionMixnet() {
    return $('[data-testid="network-mode-radio-group-option-mixnet"]');
  }
  get networkModePrivacyIcon() {
    return $('[data-testid="network-mode-privacy-icon"]');
  }
  get networkModeOptionMixnetLabel() {
    return $('[data-testid="network-mode-radio-group-option-mixnet-label"]');
  }
  get networkModeOptionMixnetDescription() {
    return $(
      '[data-testid="network-mode-radio-group-option-mixnet-description"]',
    );
  }

  // Location selection elements
  get homeNodeSelectSection() {
    return $('[data-testid="home-node-select-section"]');
  }
  get homeNodeSelectTitle() {
    return $('[data-testid="home-node-select-title"]');
  }
  get homeHopSelectsContainer() {
    return $('[data-testid="home-hop-selects-container"]');
  }

  // Entry location selector
  get hopSelectEntry() {
    return $('[data-testid="hop-select-entry"]');
  }
  get hopSelectLabelEntry() {
    return $('[data-testid="hop-select-label-entry"]');
  }
  get hopSelectCountryEntry() {
    return $('[data-testid="hop-select-country-entry"]');
  }
  get hopSelectFlagEntry() {
    return $('[data-testid="hop-select-flag-entry"]');
  }
  get hopSelectCountryNameEntry() {
    return $('[data-testid="hop-select-country-name-entry"]');
  }
  get hopSelectArrowEntry() {
    return $('[data-testid="hop-select-arrow-entry"]');
  }

  // Exit location selector
  get hopSelectExit() {
    return $('[data-testid="hop-select-exit"]');
  }
  get hopSelectLabelExit() {
    return $('[data-testid="hop-select-label-exit"]');
  }
  get hopSelectCountryExit() {
    return $('[data-testid="hop-select-country-exit"]');
  }
  get hopSelectFlagExit() {
    return $('[data-testid="hop-select-flag-exit"]');
  }
  get hopSelectCountryNameExit() {
    return $('[data-testid="hop-select-country-name-exit"]');
  }
  get hopSelectArrowExit() {
    return $('[data-testid="hop-select-arrow-exit"]');
  }

  // Main action button
  get homeConnectionButton() {
    return $('[data-testid="home-connection-button"]');
  }
  get homeConnectionButtonText() {
    return $('[data-testid="home-connection-button-text"]');
  }

  async waitForPageLoad() {
    await super.waitForPageLoad();
    await this.waitForElementTextToEqual(this.networkModeLabel, 'Select mode');
    await this.waitForElementAttributeToEqual(
      this.connectionBadge,
      'data-status',
      'Disconnected',
    );
  }

  async getCurrentConnectionStatus() {
    return await this.getElementText(this.connectionStatusText);
  }

  async openSettings() {
    await this.clickElement(this.topBarRightButton);
  }

  async selectNetworkMode(mode) {
    if (mode.toLowerCase() === 'wireguard' || mode.toLowerCase() === 'fast') {
      await this.clickElement(this.networkModeOptionWireguard);
    } else if (
      mode.toLowerCase() === 'mixnet' ||
      mode.toLowerCase() === 'anonymous'
    ) {
      await this.clickElement(this.networkModeOptionMixnet);
    } else {
      throw new Error(`Unknown network mode: ${mode}`);
    }

    await this.waitForAppUpdate();
  }

  async clickConnect() {
    await this.clickElement(this.homeConnectionButton);
    await this.waitForAppUpdate();
  }

  async getSelectedEntry() {
    return await this.getElementText(this.hopSelectCountryNameEntry);
  }

  async getSelectedExit() {
    return await this.getElementText(this.hopSelectCountryNameExit);
  }

  async getNetworkMode() {
    // Check which option is selected
    const wireguardChecked =
      await this.networkModeOptionWireguard.getAttribute('aria-checked');
    if (wireguardChecked === 'true') {
      return 'Fast (WireGuard*)';
    } else {
      return 'Anonymous (mixnet)';
    }
  }

  async selectEntryLocation(country, locationPage) {
    await this.clickElement(this.hopSelectEntry);
    await locationPage.dialogTitle.waitForDisplayed();
    await locationPage.selectLocation(country);
    await this.waitForAppUpdate();

    await this.waitForElementTextToEqual(
      this.hopSelectCountryNameEntry,
      country,
    );
  }

  async selectExitLocation(country, locationPage) {
    await this.clickElement(this.hopSelectExit);
    await locationPage.dialogTitle.waitForDisplayed();
    await locationPage.selectLocation(country);
    await this.waitForAppUpdate();

    await this.waitForElementTextToEqual(
      this.hopSelectCountryNameExit,
      country,
    );
  }

  async isConnected() {
    const status = await this.getCurrentConnectionStatus();
    return status === 'Connected';
  }

  async waitForConnectionState(expectedState, timeout) {
    const actualTimeout =
      timeout ||
      getTimeout(
        'connection',
        expectedState === 'Connected' ? 'connection' : 'disconnection',
      );

    return browser.waitUntil(
      async () => {
        const currentState = await this.getCurrentConnectionStatus();
        return currentState === expectedState;
      },
      {
        timeout: actualTimeout,
        timeoutMsg: `Expected connection state to be "${expectedState}" within ${actualTimeout}ms`,
      },
    );
  }

  async connectToVPN(entryLocation, exitLocation, locationPage, timeout) {
    const connectionTimeout = timeout || getTimeout('connection', 'connection');

    // In CI, we might want to mock the connection?? Let's check how we can stub this..
    if (isCI) {
      console.log('Running in CI environment - mocking connection');
      // TODO
    }

    if (entryLocation && locationPage) {
      await this.selectEntryLocation(entryLocation, locationPage);
    }

    if (exitLocation && locationPage) {
      await this.selectExitLocation(exitLocation, locationPage);
    }

    await this.clickConnect();

    try {
      await this.waitForConnectionState('Connected', connectionTimeout);
      return true;
    } catch (error) {
      console.error(`Failed to connect to VPN: ${error.message}`);
      return false;
    }
  }

  async disconnectFromVPN(timeout) {
    const disconnectionTimeout =
      timeout || getTimeout('connection', 'disconnection');

    if (await this.isConnected()) {
      await this.clickConnect();

      try {
        await this.waitForConnectionState('Disconnected', disconnectionTimeout);
        return true;
      } catch (error) {
        console.error(`Failed to disconnect from VPN: ${error.message}`);
        return false;
      }
    }

    return true;
  }
}

module.exports = new HomePage();
