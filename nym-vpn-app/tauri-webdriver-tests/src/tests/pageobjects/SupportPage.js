const BasePage = require('./BasePage');
const { waitForTauriRerender } = require('../utils/test-utils');

class SupportPage extends BasePage {
  get supportPage() {
    return $('[data-testid="support-page"]');
  }

  // Navigation bar elements
  get topBar() {
    return $('[data-testid="top-bar"]');
  }
  get topBarLeftButton() {
    return $('[data-testid="top-bar-left-button"]');
  }
  get topBarLeftIcon() {
    return $('[data-testid="top-bar-left-icon"]');
  }
  get topBarTitle() {
    return $('[data-testid="top-bar-title-text"]');
  }

  // FAQ button
  get faqButton() {
    return $('[data-testid="support-faq-button"]');
  }
  get faqButtonTitle() {
    return $('[data-testid="support-faq-button-title"]');
  }
  get faqButtonLeadingIcon() {
    return $('[data-testid="support-faq-button-leading-icon"]');
  }
  get faqButtonTrailingIcon() {
    return $('[data-testid="support-faq-button-trailing-icon"]');
  }

  // Contact button
  get contactButton() {
    return $('[data-testid="support-contact-button"]');
  }
  get contactButtonTitle() {
    return $('[data-testid="support-contact-button-title"]');
  }
  get contactButtonLeadingIcon() {
    return $('[data-testid="support-contact-button-leading-icon"]');
  }
  get contactButtonTrailingIcon() {
    return $('[data-testid="support-contact-button-trailing-icon"]');
  }

  // Telegram button
  get telegramButton() {
    return $('[data-testid="support-telegram-button"]');
  }
  get telegramButtonTitle() {
    return $('[data-testid="support-telegram-button-title"]');
  }
  get telegramButtonLeadingComponent() {
    return $('[data-testid="support-telegram-button-leading-component"]');
  }
  get telegramButtonTrailingIcon() {
    return $('[data-testid="support-telegram-button-trailing-icon"]');
  }

  // Matrix button
  get matrixButton() {
    return $('[data-testid="support-matrix-button"]');
  }
  get matrixButtonTitle() {
    return $('[data-testid="support-matrix-button-title"]');
  }
  get matrixButtonLeadingComponent() {
    return $('[data-testid="support-matrix-button-leading-component"]');
  }
  get matrixButtonTrailingIcon() {
    return $('[data-testid="support-matrix-button-trailing-icon"]');
  }

  // Discord button
  get discordButton() {
    return $('[data-testid="support-discord-button"]');
  }
  get discordButtonTitle() {
    return $('[data-testid="support-discord-button-title"]');
  }
  get discordButtonLeadingComponent() {
    return $('[data-testid="support-discord-button-leading-component"]');
  }
  get discordButtonTrailingIcon() {
    return $('[data-testid="support-discord-button-trailing-icon"]');
  }

  // GitHub button
  get githubButton() {
    return $('[data-testid="support-github-button"]');
  }
  get githubButtonTitle() {
    return $('[data-testid="support-github-button-title"]');
  }
  get githubButtonLeadingComponent() {
    return $('[data-testid="support-github-button-leading-component"]');
  }
  get githubButtonTrailingIcon() {
    return $('[data-testid="support-github-button-trailing-icon"]');
  }

  async waitForPageLoad() {
    await this.topBarTitle.waitForDisplayed();
    await this.waitForElementTextToEqual(this.topBarTitle, 'Support');
    await this.supportPage.waitForDisplayed();
    await waitForTauriRerender();
  }

  async goBack() {
    await this.clickElement(this.topBarLeftButton);
    await waitForTauriRerender();
  }

  // For the below, this opens an external link, we can't verify it directly
  async openFAQ() {
    await this.clickElement(this.faqButton);
    // Since this opens an external link, we can't verify it directly
    // In a real test, we might need to check for a new window or tab
  }

  async openContact() {
    await this.clickElement(this.contactButton);
  }

  async openTelegram() {
    await this.clickElement(this.telegramButton);
  }

  async openMatrix() {
    await this.clickElement(this.matrixButton);
  }

  async openDiscord() {
    await this.clickElement(this.discordButton);
  }

  async openGitHub() {
    await this.clickElement(this.githubButton);
  }

  async areSupportButtonsDisplayed() {
    const buttonsDisplayed = await Promise.all([
      this.faqButton.isDisplayed(),
      this.contactButton.isDisplayed(),
      this.telegramButton.isDisplayed(),
      this.matrixButton.isDisplayed(),
      this.discordButton.isDisplayed(),
      this.githubButton.isDisplayed(),
    ]);

    return buttonsDisplayed.every((isDisplayed) => isDisplayed === true);
  }
}

module.exports = new SupportPage();
