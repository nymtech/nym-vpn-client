const HomePage = require('../pageobjects/HomePage');

describe('NymVPN Homepage', () => {
  beforeEach(async () => {
    await HomePage.open();
  });

  describe('Initial State', () => {
    it('should display the homepage with disconnected status', async () => {
      await expect(HomePage.homeContainer).toBeDisplayed();

      const connectionStatus = await HomePage.getCurrentConnectionStatus();
      await expect(connectionStatus).toBe('Disconnected');

      await HomePage.checkElementDetails(HomePage.homeConnectionButtonText);

      await HomePage.waitForButtonText();
      const buttonText = await HomePage.homeConnectionButtonText.getText();
      await expect(buttonText).toBe('Connect');
    });

    it('should have WireGuard selected as the default network mode', async () => {
      const networkMode = await HomePage.getNetworkMode();
      await expect(networkMode).toBe('Fast (WireGuard*)');

      const isSelected =
        await HomePage.networkModeOptionWireguard.getAttribute('aria-checked');
      await expect(isSelected).toBe('true');
    });

    it('should display Switzerland as the default entry and exit location', async () => {
      await HomePage.waitForCountryText();

      const entryLocation = await HomePage.getSelectedEntry();
      console.log(entryLocation);
      await expect(entryLocation).toBe('Switzerland');

      const exitLocation = await HomePage.getSelectedExit();

      await HomePage.checkElementDetails(HomePage.hopSelectCountryNameEntry);
      console.log(exitLocation);
      await expect(exitLocation).toBe('Switzerland');
    });
  });

  describe('Network Mode Selection', () => {
    it('should allow switching between network modes', async () => {
      let networkMode = await HomePage.getNetworkMode();
      await expect(networkMode).toBe('Fast (WireGuard*)');

      await HomePage.selectNetworkMode('mixnet');

      networkMode = await HomePage.getNetworkMode();
      await expect(networkMode).toBe('Anonymous (mixnet)');

      await HomePage.selectNetworkMode('wireguard');

      networkMode = await HomePage.getNetworkMode();
      await expect(networkMode).toBe('Fast (WireGuard*)');
    });

    it.skip('should show the info popup when clicking the info button', async () => {
      await HomePage.networkModeInfoButton.click();
    });
  });

  // We want to validate that if a user tries to connect it should redirect to the user login page
  // TODO
  describe('Connection Button', () => {
    it.skip('should attempt to connect when clicking the connect button', async () => {
      let connectionStatus = await HomePage.getCurrentConnectionStatus();
      await expect(connectionStatus).toBe('Disconnected');

      await HomePage.clickConnect();
    });
  });
});
