const HomePage = require('../pageobjects/HomePage');
const SettingsPage = require('../pageobjects/SettingsPage');
const SupportPage = require('../pageobjects/SupportPage');

describe('Support Page Tests', () => {
  beforeEach(async () => {
    await HomePage.open();
    await HomePage.waitForPageLoad();

    await HomePage.openSettings();
    await SettingsPage.waitForPageLoad();

    await SettingsPage.openSupportPage();
    await SupportPage.waitForPageLoad();
  });

  afterEach(async () => {
    try {
      if (
        (await SupportPage.topBarTitle.isDisplayed()) &&
        (await SupportPage.topBarTitle.getText()) === 'Support'
      ) {
        await SupportPage.goBack();
        await SettingsPage.waitForPageLoad();
      }

      if (
        (await SettingsPage.topBarTitle.isDisplayed()) &&
        (await SettingsPage.topBarTitle.getText()) === 'Settings'
      ) {
        await SettingsPage.goBack();
        await HomePage.waitForPageLoad();
      }
    } catch (e) {
      console.log('Navigation error in cleanup:', e);
    }
  });

  it('should display the support page with correct title', async () => {
    const title = await SupportPage.topBarTitle.getText();
    expect(title).toBe('Support');
  });

  it('should display all support channel buttons', async () => {
    const allButtonsDisplayed = await SupportPage.areSupportButtonsDisplayed();
    expect(allButtonsDisplayed).toBe(true);

    expect(await SupportPage.faqButtonTitle.getText()).toBe('Check the FAQ');
    expect(await SupportPage.contactButtonTitle.getText()).toBe('Get in touch');
    expect(await SupportPage.telegramButtonTitle.getText()).toBe(
      'Chat on Telegram',
    );
    expect(await SupportPage.matrixButtonTitle.getText()).toBe(
      'Join us on Matrix',
    );
    expect(await SupportPage.discordButtonTitle.getText()).toBe(
      'Join us on Discord',
    );
    expect(await SupportPage.githubButtonTitle.getText()).toBe(
      'Open a GitHub issue',
    );
  });

  it('should navigate back to settings page when back button is clicked', async () => {
    await SupportPage.goBack();

    await SettingsPage.waitForPageLoad();
    const title = await SettingsPage.topBarTitle.getText();
    expect(title).toBe('Settings');
  });

  // The following tests are skipped as they would open external links
  // which is not easily testable in automated tests

  it.skip('should open FAQ when FAQ button is clicked', async () => {
    await SupportPage.openFAQ();
    // TODO
  });

  it.skip('should open contact form when contact button is clicked', async () => {
    await SupportPage.openContact();
    // TODOV
  });

  it.skip('should open Telegram when Telegram button is clicked', async () => {
    await SupportPage.openTelegram();
    // TODO
  });

  it.skip('should open Matrix when Matrix button is clicked', async () => {
    await SupportPage.openMatrix();
    // TODO
  });

  it.skip('should open Discord when Discord button is clicked', async () => {
    await SupportPage.openDiscord();
    // TODO
  });

  it.skip('should open GitHub when GitHub button is clicked', async () => {
    await SupportPage.openGitHub();
    // TODO
  });
});
