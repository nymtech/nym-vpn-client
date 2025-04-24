const BasePage = require('./BasePage');
const { waitForTauriRerender } = require('../utils/test-utils');

class SettingsPage extends BasePage {
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

  // Daemon status indicator
  get daemonDot() {
    return $('[data-testid="daemon-dot"]');
  }
  get daemonDotIndicator() {
    return $('[data-testid="daemon-dot-indicator"]');
  }

  // Settings cards
  get accountCard() {
    return $('[data-testid="settings-card-account"]');
  }
  get accountCardTitle() {
    return $('[data-testid="settings-card-account-title"]');
  }
  get accountCardLeadingIcon() {
    return $('[data-testid="settings-card-account-leading-icon"]');
  }
  get accountCardTrailingIcon() {
    return $('[data-testid="settings-card-account-trailing-icon"]');
  }

  // Legal card
  get legalCard() {
    return $('[data-testid="settings-card-legal"]');
  }
  get legalCardTitle() {
    return $('[data-testid="settings-card-legal-title"]');
  }
  get legalCardTrailingIcon() {
    return $('[data-testid="settings-card-legal-trailing-icon"]');
  }

  // Logout button
  get logoutButton() {
    return $('[data-testid="logout-button"]');
  }
  get logoutButtonTitle() {
    return $('[data-testid="logout-button-title"]');
  }

  // Quit NymVPN button
  get quitNymVpnButton() {
    return $('[data-testid="settings-card-quit-nymvpn"]');
  }
  get quitNymVpnButtonTitle() {
    return $('[data-testid="settings-card-quit-nymvpn-title"]');
  }

  // Switches and toggles
  get errorReportsSwitch() {
    return $('#headlessui-switch-«rh»');
  }

  get autostartSwitch() {
    return $('#headlessui-switch-«rm»');
  }

  get killswitchSwitch() {
    return $('#headlessui-switch-«rq»');
  }

  get desktopNotificationsSwitch() {
    return $('#headlessui-switch-«r12»');
  }

  // Info and about section
  get infoDataContainer() {
    return $('[data-testid="info-data-container"]');
  }

  // Client version
  get clientVersionContainer() {
    return $('[data-testid="client-version-container"]');
  }
  get clientVersionLabel() {
    return $('[data-testid="client-version-label"]');
  }
  get clientVersionValue() {
    return $('[data-testid="client-version-value"]');
  }
  get clientVersionContent() {
    return $('[data-testid="client-version-value-content"]');
  }

  // Daemon version
  get daemonVersionContainer() {
    return $('[data-testid="daemon-version-container"]');
  }
  get daemonVersionLabel() {
    return $('[data-testid="daemon-version-label"]');
  }
  get daemonVersionValue() {
    return $('[data-testid="daemon-version-value"]');
  }
  get daemonVersionContent() {
    return $('[data-testid="daemon-version-value-content"]');
  }

  // Network info
  get networkNameContainer() {
    return $('[data-testid="network-name-container"]');
  }
  get networkNameLabel() {
    return $('[data-testid="network-name-label"]');
  }
  get networkNameValue() {
    return $('[data-testid="network-name-value"]');
  }
  get networkNameContent() {
    return $('[data-testid="network-name-value-content"]');
  }

  // Account data
  get accountDataContainer() {
    return $('[data-testid="account-data-container"]');
  }

  // Account ID
  get accountIdContainer() {
    return $('[data-testid="account-id-container"]');
  }
  get accountIdLabel() {
    return $('[data-testid="account-id-label"]');
  }
  get accountIdValue() {
    return $('[data-testid="account-id-value"]');
  }
  get accountIdContent() {
    return $('[data-testid="account-id-value-content"]');
  }

  // Device ID
  get deviceIdContainer() {
    return $('[data-testid="device-id-container"]');
  }
  get deviceIdLabel() {
    return $('[data-testid="device-id-label"]');
  }
  get deviceIdValue() {
    return $('[data-testid="device-id-value"]');
  }
  get deviceIdContent() {
    return $('[data-testid="device-id-value-content"]');
  }

  async waitForPageLoad() {
    await this.topBarTitle.waitForDisplayed();
    await this.waitForElementTextToEqual(this.topBarTitle, 'Settings');
    await waitForTauriRerender();
  }

  async openAccountPage() {
    await this.clickElement(this.accountCard);
    await waitForTauriRerender();
  }

  async openLegalPage() {
    await this.clickElement(this.legalCard);
    await waitForTauriRerender();
  }

  async openSupportPage() {
    try {
      const radiogroup = await $('[role="radiogroup"]');

      if (await radiogroup.isExisting()) {
        const firstRadioOption = await radiogroup.$('span[role="radio"]');

        if (await firstRadioOption.isExisting()) {
          await this.clickElement(firstRadioOption);
          await waitForTauriRerender();
          return;
        }
      }

      console.log(
        'Could not find support option by radiogroup, trying alternative approach',
      );

      const elements = await $$('.text-base');
      for (const element of elements) {
        const text = await element.getText();
        if (text.includes('Support') || text.includes('feedback')) {
          const parentSpan = await element.$('./../../..');
          if (parentSpan && (await parentSpan.isExisting())) {
            await this.clickElement(parentSpan);
            await waitForTauriRerender();
            return;
          }
        }
      }

      const possibleElements = await $$('span[role="radio"]');
      if (possibleElements.length > 0) {
        await this.clickElement(possibleElements[0]);
        await waitForTauriRerender();
      } else {
        throw new Error('Could not find any support options in settings page');
      }
    } catch (error) {
      console.error('Failed to navigate to support page:', error.message);
      throw error;
    }
  }

  async logout() {
    await this.clickElement(this.logoutButton);
    await waitForTauriRerender(1000);
  }

  async quitApplication() {
    await this.clickElement(this.quitNymVpnButton);
    await waitForTauriRerender(1000);
  }

  async goBack() {
    await this.clickElement(this.topBarLeftButton);
    await waitForTauriRerender();
  }

  async toggleAutostart() {
    await this.clickElement(this.autostartSwitch);
    await waitForTauriRerender();
  }

  async toggleDesktopNotifications() {
    await this.clickElement(this.desktopNotificationsSwitch);
    await waitForTauriRerender();
  }

  async toggleErrorReports() {
    await this.clickElement(this.errorReportsSwitch);
    await waitForTauriRerender();
  }

  async isAutostartEnabled() {
    return (await this.autostartSwitch.getAttribute('aria-checked')) === 'true';
  }

  async areDesktopNotificationsEnabled() {
    return (
      (await this.desktopNotificationsSwitch.getAttribute('aria-checked')) ===
      'true'
    );
  }

  async areErrorReportsEnabled() {
    return (
      (await this.errorReportsSwitch.getAttribute('aria-checked')) === 'true'
    );
  }

  async getClientVersion() {
    return await this.getElementText(this.clientVersionContent);
  }

  async getDaemonVersion() {
    return await this.getElementText(this.daemonVersionContent);
  }

  async getNetworkName() {
    return await this.getElementText(this.networkNameContent);
  }

  async getAccountId() {
    return await this.getElementText(this.accountIdContent);
  }

  async getDeviceId() {
    return await this.getElementText(this.deviceIdContent);
  }

  async getDaemonStatus() {
    return await this.daemonDot.getAttribute('data-status');
  }
}

module.exports = new SettingsPage();
