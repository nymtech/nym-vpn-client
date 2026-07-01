import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { Gateway, GatewaysByCountry } from '../../types';

import { renderWithProviders, seedStore } from '../../test/harness';
import { initialState } from '../../store/slices/createMainSlice';
import { useNodeListStateStore } from '../../store/nodeListState';
import NodeLocation from './NodeLocation';

// `NodeLocation` renders `Node`, which transitively imports the `../../ui`
// barrel loading `DaemonDot` (reads `window._APP.devMode`) and the Tauri OS
// plugin at module-load time; `vi.hoisted`/`vi.mock` run before the import.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

// `Node` uses `useToast`, which needs a base-ui Toast provider; stub it while
// keeping every other hook real.
vi.mock('../../hooks', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../hooks')>();
  return { ...actual, useToast: () => ({ add: vi.fn(), close: vi.fn() }) };
});

function gateway(id: string, name: string): Gateway {
  return {
    id,
    type: 'wg',
    name,
    country: { code: 'DE', name: 'Germany' },
    location: { latitude: 0, longitude: 0, city: 'Berlin', region: '' },
    description: null,
    asn: null,
    mxScore: 'high',
    wgScore: 'high',
    wgPerformance: null,
    exitIpv4: null,
    exitIpv6: null,
    buildVersion: null,
    quic: false,
    nodeFamilyName: null,
  } as unknown as Gateway;
}

function country(code: string, name: string): GatewaysByCountry {
  return {
    country: { code, name },
    regions: [],
    gateways: [gateway(`${code}-g1`, `${name} gw`)],
    type: 'wg',
    quic: false,
  } as unknown as GatewaysByCountry;
}

function seedExplicit() {
  seedStore({
    ...initialState,
    vpnMode: 'wg',
    daemonStatus: 'down',
    wg: [country('DE', 'Germany')],
    mxEntry: [],
    mxExit: [],
    wgLoading: false,
    mxEntryLoading: false,
    mxExitLoading: false,
    wgError: null,
    mxEntryError: null,
    mxExitError: null,
    gatewaySelectionAlgorithmConfig: {
      enableGeoLocation: true,
      gatewaySelectionAlgorithm: 'explicit',
    },
  });
}

afterEach(() => {
  useNodeListStateStore.getState().reset('all');
});

describe('NodeLocation', () => {
  it('shows both entry and exit tabs in the explicit algorithm', () => {
    seedExplicit();

    renderWithProviders(<NodeLocation />);

    expect(screen.getByRole('tab', { name: 'Entry' })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'Exit' })).toBeInTheDocument();
  });

  it('hides the entry tab in the auto algorithm', () => {
    seedExplicit();
    seedStore({
      gatewaySelectionAlgorithmConfig: {
        enableGeoLocation: true,
        gatewaySelectionAlgorithm: 'auto',
      },
    });

    renderWithProviders(<NodeLocation />);

    expect(
      screen.queryByRole('tab', { name: 'Entry' }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'Exit' })).toBeInTheDocument();
  });

  it('renders the exit node panel by default', () => {
    seedExplicit();

    renderWithProviders(<NodeLocation />);

    expect(screen.getByTestId('node-container-exit')).toBeInTheDocument();
  });

  it('switches to the entry tab on click', async () => {
    const user = userEvent.setup();
    seedExplicit();

    renderWithProviders(<NodeLocation />);

    await user.click(screen.getByRole('tab', { name: 'Entry' }));

    expect(screen.getByTestId('node-container-entry')).toBeInTheDocument();
  });
});
