import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { UiGateway } from '../../../types/node';

import {
  mockTauriCommands,
  renderWithProviders,
  seedStore,
} from '../../../test/harness';
import { useNodeListStateStore } from '../../../store/nodeListState';
import NodeDetails from './NodeDetails';

// `NodeDetails` transitively imports the `../../../ui` barrel, which loads
// `DaemonDot` (reads `window._APP.devMode`) and the Tauri OS plugin at
// module-load time; `vi.hoisted`/`vi.mock` run before the static import.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

// Stub navigation so a successful select does not strip the route state (this
// component is rendered directly, not through the route tree) and re-crash on
// a null `location.state` re-render.
const navigate = vi.fn();
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useNavigate: () => navigate };
});

// `useToast` needs a base-ui Toast provider and `useClipboard` calls Tauri; stub
// both while keeping the score/lang hooks real (they drive rendered labels).
const addToast = vi.fn();
vi.mock('../../../hooks', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../hooks')>();
  return {
    ...actual,
    useToast: () => ({ add: addToast, close: vi.fn() }),
    useClipboard: () => ({ copy: vi.fn(), copied: false }),
  };
});

function gateway(overrides: Partial<UiGateway> = {}): UiGateway {
  return {
    id: 'identity-key-123',
    type: 'wg',
    name: 'gateway one',
    country: { code: 'DE', name: 'Germany' },
    location: { latitude: 0, longitude: 0, city: 'Berlin', region: '' },
    description: 'a fine gateway',
    asn: { asn: '64500', name: 'ExampleISP', type: 'residential' },
    mxScore: 'high',
    wgScore: 'high',
    wgPerformance: null,
    exitIpv4: '1.2.3.4',
    exitIpv6: null,
    buildVersion: 'v1.2.3',
    quic: true,
    nodeFamilyName: null,
    nodeType: 'gateway',
    isSelected: false,
    ...overrides,
  };
}

function renderDetails(gw: UiGateway, hop: 'entry' | 'exit' = 'exit') {
  // MemoryRouter accepts location objects, but the harness types
  // `initialEntries` as `string[]`; cast to carry route state through.
  const entries = [
    { pathname: '/node-details', state: { gateway: gw, hop } },
  ] as unknown as string[];
  return renderWithProviders(<NodeDetails />, { initialEntries: entries });
}

afterEach(() => {
  addToast.mockClear();
  navigate.mockClear();
  useNodeListStateStore.getState().reset('all');
  seedStore({
    entryNode: 'random',
    exitNode: 'random',
    backendFlags: {
      quic: false,
      domainFronting: false,
      zknymCredential: false,
    },
    quic: false,
    gatewaySelectionAlgorithmConfig: {
      enableGeoLocation: true,
      gatewaySelectionAlgorithm: 'explicit',
    },
  });
});

describe('NodeDetails', () => {
  it('renders the gateway name, location and description', () => {
    renderDetails(gateway());

    expect(screen.getByText('gateway one')).toBeInTheDocument();
    expect(screen.getByText('Berlin, Germany')).toBeInTheDocument();
    expect(screen.getByText('a fine gateway')).toBeInTheDocument();
  });

  it('renders the overall performance label from the score', () => {
    renderDetails(gateway({ wgScore: 'high' }));

    // useScore maps "high" performance to the "Good" label.
    expect(screen.getByText('Good')).toBeInTheDocument();
  });

  it('shows the residential IP feature for a residential ASN', () => {
    renderDetails(gateway());

    expect(screen.getByText('Residential')).toBeInTheDocument();
  });

  it('shows the identity key and connection details', () => {
    renderDetails(gateway());

    expect(screen.getByText('identity-key-123')).toBeInTheDocument();
    expect(screen.getByText('1.2.3.4')).toBeInTheDocument();
  });

  it('offers the select button for an unselected gateway and calls set_node', async () => {
    const user = userEvent.setup();
    const invoked: string[] = [];
    mockTauriCommands((cmd) => {
      invoked.push(cmd);
      return null;
    });

    renderDetails(gateway());

    const selectButton = screen.getByRole('button', { name: 'Select server' });
    await user.click(selectButton);

    expect(invoked).toContain('set_node');
  });

  it('hides the select button when the gateway is already selected', () => {
    seedStore({ exitNode: { gateway: { id: 'identity-key-123' } } });

    renderDetails(gateway());

    expect(
      screen.queryByRole('button', { name: 'Select server' }),
    ).not.toBeInTheDocument();
  });
});
