import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { Toast } from '@base-ui/react';
import { useAppStore } from '../../../store';
import { renderWithProviders, seedStore } from '../../../test/harness';
import GeoExclusion from './GeoExclusion';

// The screen pulls UI from the `../../../ui` barrel, which loads `DaemonDot`
// reading `window._APP.devMode` and calls the Tauri OS plugin's `type()` at
// module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const navigate = vi.fn();
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useNavigate: () => navigate };
});

afterEach(() => {
  navigate.mockReset();
  seedStore({
    geoExclusion: {
      enabled: false,
      listenPort: 1080,
      excludedCountries: ['CN'],
    },
  });
});

describe('GeoExclusion', () => {
  it('shows the description and hides the detail cards when disabled', () => {
    seedStore({
      geoExclusion: {
        enabled: false,
        listenPort: 1080,
        excludedCountries: ['CN'],
      },
    });
    renderWithProviders(
      <Toast.Provider>
        <GeoExclusion />
      </Toast.Provider>,
    );

    expect(
      screen.getByText(/Routes traffic for selected regions/i),
    ).toBeInTheDocument();
    expect(screen.queryByText('Excluded regions')).not.toBeInTheDocument();
    expect(screen.getByRole('switch')).toHaveAttribute('aria-checked', 'false');
  });

  it('reveals the port card, regions and warning when enabled', () => {
    seedStore({
      geoExclusion: {
        enabled: true,
        listenPort: 1080,
        excludedCountries: ['CN'],
      },
    });
    renderWithProviders(
      <Toast.Provider>
        <GeoExclusion />
      </Toast.Provider>,
    );

    expect(
      screen.getByText(/Traffic to excluded regions bypasses the VPN/i),
    ).toBeInTheDocument();
    expect(screen.getByText('Excluded regions')).toBeInTheDocument();
    expect(screen.getByText('SOCKS5 port')).toBeInTheDocument();
  });

  it('toggles geo exclusion on and invokes set_geo_exclusion_enabled', async () => {
    const user = userEvent.setup();
    const calls: { cmd: string; payload?: unknown }[] = [];
    mockIPC((cmd, payload) => {
      calls.push({ cmd, payload });
      return undefined;
    });
    seedStore({
      geoExclusion: {
        enabled: false,
        listenPort: 1080,
        excludedCountries: ['CN'],
      },
    });
    renderWithProviders(
      <Toast.Provider>
        <GeoExclusion />
      </Toast.Provider>,
    );

    await user.click(screen.getByRole('switch'));

    expect(calls).toContainEqual({
      cmd: 'set_geo_exclusion_enabled',
      payload: { enabled: true },
    });
    expect(useAppStore.getState().geoExclusion.enabled).toBe(true);
  });
});
