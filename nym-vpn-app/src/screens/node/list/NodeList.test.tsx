import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { UiGateway, UiGatewaysByCountry } from '../../../types/node';

import { renderWithProviders } from '../../../test/harness';
import { useNodeListStateStore } from '../../../store/nodeListState';
import NodeList, { type NodeListProps } from './NodeList';

// `NodeList` transitively imports the `../../../ui` barrel, which loads
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

function gateway(id: string, name: string): UiGateway {
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
    nodeType: 'gateway',
    isSelected: false,
  };
}

function country(
  code: string,
  name: string,
  gateways: UiGateway[],
): UiGatewaysByCountry {
  return {
    country: { code, name, nodeType: 'country', isSelected: false },
    regions: [],
    gateways,
    type: 'wg',
    i18n: name,
    isSelected: false,
  };
}

function setup(overrides: Partial<NodeListProps> = {}) {
  const props: NodeListProps = {
    nodes: [],
    gateways: [],
    onSelect: vi.fn(),
    onNodeDetails: vi.fn(),
    hop: 'entry',
    vpnMode: 'wg',
    quicFilter: false,
    expanded: [],
    focused: null,
    ...overrides,
  };
  renderWithProviders(<NodeList {...props} />);
  return props;
}

afterEach(() => {
  useNodeListStateStore.getState().reset('all');
});

describe('NodeList', () => {
  it('shows the no-results message when there are no nodes or gateways', () => {
    setup();

    expect(screen.getByText('No results found.')).toBeInTheDocument();
    expect(screen.queryByTestId('node-list-accordion')).not.toBeInTheDocument();
  });

  it('renders a country accordion row when nodes are present', () => {
    setup({ nodes: [country('DE', 'Germany', [gateway('g1', 'gw-one')])] });

    expect(screen.getByTestId('node-list-accordion')).toBeInTheDocument();
    expect(screen.getByTestId('country-name-DE')).toHaveTextContent('Germany');
  });

  it('renders the standalone search-result gateways section', () => {
    setup({ gateways: [gateway('g9', 'standalone-gw')] });

    expect(
      screen.getByTestId('standalone-gateways-container'),
    ).toBeInTheDocument();
    expect(screen.getByText('standalone-gw')).toBeInTheDocument();
  });

  it('expands a collapsed country when its fold trigger is clicked', async () => {
    const user = userEvent.setup();
    setup({ nodes: [country('DE', 'Germany', [gateway('g1', 'gw-one')])] });

    // Collapsed → the panel content is not mounted yet.
    expect(screen.queryByText('gw-one')).not.toBeInTheDocument();

    await user.click(screen.getByTestId('icon-keyboard_arrow_down'));

    // The list writes the newly expanded country into the shared store.
    expect(useNodeListStateStore.getState().entry.expanded).toContain('DE');
  });
});
