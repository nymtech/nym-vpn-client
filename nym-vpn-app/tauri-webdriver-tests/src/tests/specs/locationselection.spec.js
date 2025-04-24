const HomePage = require('../pageobjects/HomePage');
const Location = require('../pageobjects/Location');
const {
  waitAndClick,
  waitForInteractable,
  waitAndGetText,
  retry,
  waitForTextToEqual,
  waitForTauriRerender,
  isSelected,
} = require('../utils/test-utils');

describe('NymVPN Location Selection', () => {
  beforeEach(async () => {
    await HomePage.open();
    await waitForTauriRerender();
  });

  describe('Location Selectors', () => {
    it('should display the entry and exit location selectors', async () => {
      await waitForInteractable(HomePage.hopSelectEntry);
      await waitForInteractable(HomePage.hopSelectExit);

      const entryLabel = await waitAndGetText(HomePage.hopSelectLabelEntry);
      await expect(entryLabel).toBe('Entry location');

      const exitLabel = await waitAndGetText(HomePage.hopSelectLabelExit);
      await expect(exitLabel).toBe('Exit location');
    });

    it('should show location selection dialog when clicking entry location', async () => {
      await waitAndClick(HomePage.hopSelectEntry);

      await waitForInteractable(Location.dialogTitle);
      await waitForInteractable(Location.locationList);

      await waitAndClick(Location.closeButton);
    });

    it('should show location selection dialog when clicking exit location', async () => {
      await waitAndClick(HomePage.hopSelectExit);

      await waitForInteractable(Location.dialogTitle);
      await waitForInteractable(Location.locationList);

      await waitAndClick(Location.closeButton);
    });
  });

  describe('Location Selection Functionality', () => {
    it('should allow selecting a different entry location', async () => {
      await retry(async () => {
        await waitAndClick(HomePage.hopSelectEntry);

        await waitForInteractable(Location.dialogTitle);

        await waitAndClick(
          $('[data-testid="location-item"][data-country="Germany"]'),
        );

        await waitForTauriRerender();

        await waitForTextToEqual(HomePage.hopSelectCountryNameEntry, 'Germany');
      });
    });

    it('should allow selecting a different exit location', async () => {
      await retry(async () => {
        await waitAndClick(HomePage.hopSelectExit);

        await waitForInteractable(Location.dialogTitle);

        await waitAndClick(
          $('[data-testid="location-item"][data-country="Netherlands"]'),
        );

        await waitForTauriRerender();

        await waitForTextToEqual(
          HomePage.hopSelectCountryNameExit,
          'Netherlands',
        );
      });
    });

    it('should allow searching for locations', async () => {
      await retry(async () => {
        await waitAndClick(HomePage.hopSelectEntry);

        await waitForInteractable(Location.dialogTitle);

        await Location.searchForLocation('Canada');

        await waitForTauriRerender();

        const canadaElement = await $(
          '[data-testid="location-item"][data-country="Canada"]',
        );
        await waitForInteractable(canadaElement);

        await waitAndClick(canadaElement);

        await waitForTauriRerender();

        await waitForTextToEqual(HomePage.hopSelectCountryNameEntry, 'Canada');
      });
    });

    it('should remember location selections when toggling between network modes', async () => {
      await retry(async () => {
        await waitAndClick(HomePage.hopSelectEntry);
        await waitForInteractable(Location.dialogTitle);
        await waitAndClick(
          $('[data-testid="location-item"][data-country="Germany"]'),
        );
        await waitForTauriRerender();

        await waitAndClick(HomePage.hopSelectExit);
        await waitForInteractable(Location.dialogTitle);
        await waitAndClick(
          $('[data-testid="location-item"][data-country="Netherlands"]'),
        );
        await waitForTauriRerender();

        await waitForTextToEqual(HomePage.hopSelectCountryNameEntry, 'Germany');
        await waitForTextToEqual(
          HomePage.hopSelectCountryNameExit,
          'Netherlands',
        );

        await waitAndClick(HomePage.networkModeOptionMixnet);
        await waitForTauriRerender();

        const isMixnetSelected = await isSelected(
          HomePage.networkModeOptionMixnet,
        );
        expect(isMixnetSelected).toBe(true);

        await waitAndClick(HomePage.networkModeOptionWireguard);
        await waitForTauriRerender();

        const isWireGuardSelected = await isSelected(
          HomePage.networkModeOptionWireguard,
        );
        expect(isWireGuardSelected).toBe(true);

        await waitForTextToEqual(HomePage.hopSelectCountryNameEntry, 'Germany');
        await waitForTextToEqual(
          HomePage.hopSelectCountryNameExit,
          'Netherlands',
        );
      });
    });

    it('should allow selecting the same country for both entry and exit', async () => {
      await retry(async () => {
        await waitAndClick(HomePage.hopSelectEntry);
        await waitForInteractable(Location.dialogTitle);
        await waitAndClick(
          $('[data-testid="location-item"][data-country="France"]'),
        );
        await waitForTauriRerender();

        await waitAndClick(HomePage.hopSelectExit);
        await waitForInteractable(Location.dialogTitle);
        await waitAndClick(
          $('[data-testid="location-item"][data-country="France"]'),
        );
        await waitForTauriRerender();

        await waitForTextToEqual(HomePage.hopSelectCountryNameEntry, 'France');
        await waitForTextToEqual(HomePage.hopSelectCountryNameExit, 'France');
      });
    });
  });
});
