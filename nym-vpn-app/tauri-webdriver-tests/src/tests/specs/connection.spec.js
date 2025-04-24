const HomePage = require('../pageobjects/HomePage');
const Location = require('../pageobjects/Location');
const {
  retry,
  takeScreenshot,
  waitForTauriRerender,
} = require('../utils/test-utils');
const {
  isCI,
  conditionalTest,
  conditionalDescribe,
  getTimeout,
} = require('../utils/environment-config');

conditionalDescribe('connection')('NymVPN Connection Flow', () => {
  beforeEach(async () => {
    await HomePage.open();
    if (await HomePage.isConnected()) {
      await HomePage.disconnectFromVPN();
    }
  });

  afterEach(async () => {
    if (await HomePage.isConnected()) {
      await HomePage.disconnectFromVPN();
    }
  });

  describe('Basic Connection Tests', () => {
    it('should connect to the default location', async () => {
      const connectionTimeout = getTimeout('connection', 'connection');

      await takeScreenshot('before-connect');

      const initialStatus = await HomePage.getCurrentConnectionStatus();
      expect(initialStatus).toBe('Disconnected');

      await HomePage.clickConnect();

      await HomePage.waitForConnectionState('Connected', connectionTimeout);

      await takeScreenshot('after-connect');

      const connectedStatus = await HomePage.getCurrentConnectionStatus();
      expect(connectedStatus).toBe('Connected');

      const buttonText = await HomePage.homeConnectionButtonText.getText();
      expect(buttonText).toBe('Disconnect');
    });

    it('should disconnect from a connected state', async () => {
      const connectionTimeout = getTimeout('connection', 'connection');
      const disconnectionTimeout = getTimeout('connection', 'disconnection');

      await HomePage.connectToVPN(null, null, null, connectionTimeout);

      expect(await HomePage.getCurrentConnectionStatus()).toBe('Connected');

      await HomePage.clickConnect();

      await HomePage.waitForConnectionState(
        'Disconnected',
        disconnectionTimeout,
      );

      const disconnectedStatus = await HomePage.getCurrentConnectionStatus();
      expect(disconnectedStatus).toBe('Disconnected');

      const buttonText = await HomePage.homeConnectionButtonText.getText();
      expect(buttonText).toBe('Connect');
    });
  });

  describe('Custom Location Connection Tests', () => {
    it('should connect to custom entry and exit locations', async () => {
      const connectionTimeout = getTimeout('connection', 'connection');

      await retry(async () => {
        await HomePage.hopSelectEntry.click();
        await Location.dialogTitle.waitForDisplayed();

        const countries = await Location.getAvailableCountries();
        let entryCountry = 'Germany';

        for (const country of countries) {
          if (country !== 'Switzerland') {
            entryCountry = country;
            break;
          }
        }

        await Location.selectLocation(entryCountry);
        await waitForTauriRerender();

        await HomePage.hopSelectExit.click();
        await Location.dialogTitle.waitForDisplayed();

        let exitCountry = 'France';
        for (const country of countries) {
          if (country !== entryCountry && country !== 'Switzerland') {
            exitCountry = country;
            break;
          }
        }

        await Location.selectLocation(exitCountry);
        await waitForTauriRerender();

        expect(await HomePage.getSelectedEntry()).toBe(entryCountry);
        expect(await HomePage.getSelectedExit()).toBe(exitCountry);

        await HomePage.clickConnect();

        await HomePage.waitForConnectionState('Connected', connectionTimeout);

        expect(await HomePage.getCurrentConnectionStatus()).toBe('Connected');
      });
    });

    it('should connect with Mixnet mode', async () => {
      const mixnetTimeout = getTimeout('connection', 'connection') * 1.5;

      await HomePage.selectNetworkMode('mixnet');

      expect(await HomePage.getNetworkMode()).toBe('Anonymous (mixnet)');

      await HomePage.clickConnect();

      await HomePage.waitForConnectionState('Connected', mixnetTimeout);

      expect(await HomePage.getCurrentConnectionStatus()).toBe('Connected');
    });
  });

  describe('Connection Performance Tests', () => {
    it('should connect within an acceptable time window', async () => {
      const maxConnectionTime = getTimeout('connection', 'connection');

      const startTime = Date.now();

      await HomePage.clickConnect();

      await HomePage.waitForConnectionState('Connected', maxConnectionTime);

      const endTime = Date.now();
      const connectionTime = endTime - startTime;

      console.log(`Connection time: ${connectionTime}ms`);

      expect(connectionTime).toBeLessThanOrEqual(maxConnectionTime);
    });
  });
});

describe('NymVPN Connection UI', () => {
  beforeEach(async () => {
    await HomePage.open();
  });

  it('should display the connection button with correct text', async () => {
    await HomePage.homeConnectionButton.waitForDisplayed();
    const buttonText = await HomePage.homeConnectionButtonText.getText();
    expect(buttonText).toBe('Connect');
  });

  it('should display the correct initial connection status', async () => {
    const statusText = await HomePage.connectionStatusText.getText();
    expect(statusText).toBe('Disconnected');
  });
});
