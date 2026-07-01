import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { Toast } from '@base-ui/react';
import { renderWithProviders, seedStore } from '../../../test/harness';
import SetupInstructions from './SetupInstructions';

// The screen pulls UI from the `../../../ui` barrel, which loads `DaemonDot`
// reading `window._APP.devMode` and calls the Tauri OS plugin's `type()` at
// module-load time; the mocked `type()` also selects the platform section.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

afterEach(() => {
  seedStore({
    geoExclusion: {
      enabled: true,
      listenPort: 1080,
      excludedCountries: ['CN'],
    },
  });
});

describe('SetupInstructions', () => {
  it('renders the proxy address using the current listen port', () => {
    seedStore({
      geoExclusion: {
        enabled: true,
        listenPort: 4242,
        excludedCountries: ['CN'],
      },
    });
    renderWithProviders(
      <Toast.Provider>
        <SetupInstructions />
      </Toast.Provider>,
    );

    expect(screen.getByText('Proxy address')).toBeInTheDocument();
    expect(screen.getByText('127.0.0.1:4242')).toBeInTheDocument();
  });

  it('interpolates the listen port into the platform setup steps', () => {
    seedStore({
      geoExclusion: {
        enabled: true,
        listenPort: 4242,
        excludedCountries: ['CN'],
      },
    });
    renderWithProviders(
      <Toast.Provider>
        <SetupInstructions />
      </Toast.Provider>,
    );

    // Every numbered step is rendered; the Linux section references the port.
    expect(screen.getAllByText(/4242/).length).toBeGreaterThan(1);
  });
});
