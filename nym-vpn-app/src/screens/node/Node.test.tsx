import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { AppError, Gateway, GatewaysByCountry } from '../../types';

import { renderWithProviders, seedStore } from '../../test/harness';
import { initialState } from '../../store/slices/createMainSlice';
import { useNodeListStateStore } from '../../store/nodeListState';
import Node from './Node';

// `Node` transitively imports the `../../ui` barrel, which loads `DaemonDot`
// (reads `window._APP.devMode`) and the Tauri OS plugin at module-load time;
// `vi.hoisted`/`vi.mock` run before the static import.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

// `useToast` needs a base-ui Toast provider; stub it while keeping every other
// hook (notably `useI18nError` and `useNodeListData`) real.
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

afterEach(() => {
  useNodeListStateStore.getState().reset('all');
  seedStore({
    ...initialState,
    wg: [],
    mxEntry: [],
    mxExit: [],
    wgLoading: false,
    mxEntryLoading: false,
    mxExitLoading: false,
    wgError: null,
    mxEntryError: null,
    mxExitError: null,
  });
});

describe('Node', () => {
  it('renders the search input and the random quick-pick', () => {
    seedStore({
      vpnMode: 'wg',
      daemonStatus: 'down',
      wg: [country('DE', 'Germany')],
    });

    renderWithProviders(<Node node="entry" />);

    expect(screen.getByPlaceholderText('Search location')).toBeInTheDocument();
    expect(screen.getByText('Random')).toBeInTheDocument();
    expect(screen.getByTestId('node-container-entry')).toBeInTheDocument();
  });

  it('renders the node list populated from the store', () => {
    seedStore({
      vpnMode: 'wg',
      daemonStatus: 'down',
      wg: [country('DE', 'Germany')],
    });

    renderWithProviders(<Node node="entry" />);

    expect(screen.getByTestId('node-list-accordion')).toBeInTheDocument();
    expect(screen.getByTestId('country-name-DE')).toBeInTheDocument();
  });

  it('shows the loading indicator while the list is empty and loading', () => {
    seedStore({
      vpnMode: 'wg',
      daemonStatus: 'down',
      wg: [],
      wgLoading: true,
    });

    renderWithProviders(<Node node="entry" />);

    expect(screen.getByTestId('node-loading-indicator')).toBeInTheDocument();
    expect(screen.queryByText('Random')).not.toBeInTheDocument();
  });

  it('renders the error view when the active list has an error', () => {
    const error = {
      key: 'unknown',
      message: 'boom',
    } as unknown as AppError;
    seedStore({ vpnMode: 'wg', daemonStatus: 'down', wgError: error });

    renderWithProviders(<Node node="entry" />);

    expect(screen.getByTestId('node-error-container')).toBeInTheDocument();
    expect(screen.getByTestId('node-error-title')).toHaveTextContent(
      'An error occurred',
    );
  });

  it('lets the user type into the search field', async () => {
    const user = userEvent.setup();
    seedStore({
      vpnMode: 'wg',
      daemonStatus: 'down',
      wg: [country('DE', 'Germany')],
    });

    renderWithProviders(<Node node="entry" />);

    const input = screen.getByPlaceholderText('Search location');
    await user.type(input, 'Ger');

    expect(useNodeListStateStore.getState().entry.search).toBe('Ger');
  });
});
