import { Collapsible } from '@base-ui-components/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { UiGateway, UiGatewaysByCountry } from '../../../types/node';

import { renderWithProviders } from '../../../test/harness';
import { useNodeListStateStore } from '../../../store/nodeListState';
import { NodeItem } from './NodeItem';

// `NodeItem` transitively imports the `../../../ui` barrel, which loads
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

function renderItem(
  node: UiGatewaysByCountry,
  extra: Partial<React.ComponentProps<typeof NodeItem>> = {},
) {
  const props: React.ComponentProps<typeof NodeItem> = {
    node,
    hop: 'entry',
    vpnMode: 'wg',
    quicFilter: false,
    expanded: [node.country.code],
    onExpandChange: vi.fn(),
    handleLocationSelect: vi.fn(),
    onGatewaySelect: vi.fn(),
    onNodeDetails: vi.fn(),
    ...extra,
  };
  return renderWithProviders(
    <Collapsible.Root open>
      <NodeItem {...props} />
    </Collapsible.Root>,
  );
}

afterEach(() => {
  useNodeListStateStore.getState().reset('all');
});

describe('NodeItem', () => {
  it('renders the country header and its gateways when expanded', () => {
    renderItem(
      country('DE', 'Germany', [
        gateway('g1', 'gw-one'),
        gateway('g2', 'gw-two'),
      ]),
    );

    expect(screen.getByTestId('country-name-DE')).toHaveTextContent('Germany');
    expect(screen.getByText('gw-one')).toBeInTheDocument();
    expect(screen.getByText('gw-two')).toBeInTheDocument();
  });

  it('selects the country from the row header click', async () => {
    const handleLocationSelect = vi.fn();
    const user = userEvent.setup();
    renderItem(country('DE', 'Germany', [gateway('g1', 'gw-one')]), {
      handleLocationSelect,
    });

    await user.click(screen.getByTestId('country-name-DE'));

    expect(handleLocationSelect).toHaveBeenCalledOnce();
  });

  it('forwards a gateway selection through onGatewaySelect', async () => {
    const onGatewaySelect = vi.fn();
    const user = userEvent.setup();
    renderItem(country('DE', 'Germany', [gateway('g1', 'gw-one')]), {
      onGatewaySelect,
    });

    await user.click(screen.getByText('gw-one'));

    expect(onGatewaySelect).toHaveBeenCalledOnce();
  });
});
