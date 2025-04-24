const HomePage = require('../pageobjects/HomePage');
const SettingsPage = require('../pageobjects/SettingsPage');
const SupportPage = require('../pageobjects/SupportPage');

describe('Settings Page Tests', () => {
  beforeEach(async () => {
    await HomePage.open();
    await HomePage.waitForPageLoad();

    await HomePage.openSettings();
    await SettingsPage.waitForPageLoad();
  });

  afterEach(async () => {
    try {
      if (await browser.$('[data-testid="support-page"]').isDisplayed()) {
        await SupportPage.goBack();
        await SettingsPage.waitForPageLoad();
      }

      if (await SettingsPage.topBarTitle.isDisplayed()) {
        await SettingsPage.goBack();
        await HomePage.waitForPageLoad();
      }
    } catch (e) {
      console.log('Already on home page or other page', e);
    }
  });

  it('should display the settings page with correct title', async () => {
    const title = await SettingsPage.topBarTitle.getText();
    expect(title).toBe('Settings');
  });

  it('should display the app version information', async () => {
    await SettingsPage.clientVersionContainer.waitForDisplayed();
    const version = await SettingsPage.getClientVersion();
    expect(version).not.toBe(null);
    expect(version.length).toBeGreaterThan(0);
  });

  it('should display the daemon version information', async () => {
    await SettingsPage.daemonVersionContainer.waitForDisplayed();
    const version = await SettingsPage.getDaemonVersion();
    expect(version).not.toBe(null);
    expect(version.length).toBeGreaterThan(0);
  });

  it('should toggle autostart setting', async () => {
    const initialState = await SettingsPage.isAutostartEnabled();

    await SettingsPage.toggleAutostart();

    const newState = await SettingsPage.isAutostartEnabled();

    expect(newState).not.toBe(initialState);

    await SettingsPage.toggleAutostart();

    const finalState = await SettingsPage.isAutostartEnabled();
    expect(finalState).toBe(initialState);
  });

  it('should toggle desktop notifications setting', async () => {
    const initialState = await SettingsPage.areDesktopNotificationsEnabled();

    await SettingsPage.toggleDesktopNotifications();

    const newState = await SettingsPage.areDesktopNotificationsEnabled();

    expect(newState).not.toBe(initialState);

    await SettingsPage.toggleDesktopNotifications();

    const finalState = await SettingsPage.areDesktopNotificationsEnabled();
    expect(finalState).toBe(initialState);
  });

  it('should navigate back to home page when back button is clicked', async () => {
    await SettingsPage.goBack();

    await HomePage.waitForPageLoad();
    expect(await HomePage.homeContainer.isDisplayed()).toBe(true);
  });

  it('should navigate to support page when support card is clicked', async () => {
    await SettingsPage.openSupportPage();

    await SupportPage.waitForPageLoad();
    const title = await SupportPage.topBarTitle.getText();
    expect(title).toBe('Support');

    expect(await SupportPage.areSupportButtonsDisplayed()).toBe(true);
  });

  // Skip tests that would quit the app or log out in automated testing
  it.skip('should open account page when account card is clicked', async () => {
    await SettingsPage.openAccountPage();
    // TODO
  });

  it.skip('should open legal page when legal card is clicked', async () => {
    await SettingsPage.openLegalPage();
    // TODO
  });
});
