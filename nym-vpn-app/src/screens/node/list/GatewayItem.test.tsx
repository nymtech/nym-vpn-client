import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { UiGateway } from '../../../types/node';

import { renderWithProviders } from '../../../test/harness';
import { useNodeListStateStore } from '../../../store/nodeListState';
import GatewayItem from './GatewayItem';

// `GatewayItem` imports `ButtonIcon`/`MsIcon` from the `../../../ui` barrel,
// which loads `DaemonDot` (reads `window._APP.devMode`) and the Tauri OS plugin
// at module-load time; `vi.hoisted`/`vi.mock` run before the static import.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

function gateway(overrides: Partial<UiGateway> = {}): UiGateway {
  return {
    id: 'gw-1',
    type: 'wg',
    name: 'gateway one',
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
    ...overrides,
  };
}

afterEach(() => {
  useNodeListStateStore.getState().reset('all');
});

describe('GatewayItem', () => {
  it('renders the gateway name and city', () => {
    renderWithProviders(
      <GatewayItem
        gateway={gateway()}
        node="entry"
        vpnMode="wg"
        quicLabel={false}
        onSelect={vi.fn()}
        onNodeDetails={vi.fn()}
      />,
    );

    expect(screen.getByText('gateway one')).toBeInTheDocument();
    expect(screen.getByText('Berlin')).toBeInTheDocument();
  });

  it('shows the QUIC tag only when the label is enabled and the gateway supports it', () => {
    const { rerender } = renderWithProviders(
      <GatewayItem
        gateway={gateway({ quic: true })}
        node="entry"
        vpnMode="wg"
        quicLabel
        onSelect={vi.fn()}
        onNodeDetails={vi.fn()}
      />,
    );

    expect(screen.getByText('QUIC')).toBeInTheDocument();

    rerender(
      <GatewayItem
        gateway={gateway({ quic: false })}
        node="entry"
        vpnMode="wg"
        quicLabel
        onSelect={vi.fn()}
        onNodeDetails={vi.fn()}
      />,
    );

    expect(screen.queryByText('QUIC')).not.toBeInTheDocument();
  });

  it('shows the streaming icon for a residential exit gateway', () => {
    renderWithProviders(
      <GatewayItem
        gateway={gateway({
          asn: { asn: '1', name: 'isp', type: 'residential' },
        })}
        node="exit"
        vpnMode="wg"
        quicLabel={false}
        onSelect={vi.fn()}
        onNodeDetails={vi.fn()}
      />,
    );

    expect(screen.getByTestId('icon-smart_display')).toBeInTheDocument();
  });

  it('resolves the full location string in a search result', () => {
    renderWithProviders(
      <GatewayItem
        gateway={gateway()}
        node="entry"
        vpnMode="wg"
        quicLabel={false}
        inSearchResult
        onSelect={vi.fn()}
        onNodeDetails={vi.fn()}
      />,
    );

    // Non-region country → "<city>, <countryName>".
    expect(screen.getByText('Berlin, Germany')).toBeInTheDocument();
  });

  it('calls onSelect when an unselected gateway is clicked', async () => {
    const onSelect = vi.fn();
    const user = userEvent.setup();
    renderWithProviders(
      <GatewayItem
        gateway={gateway()}
        node="entry"
        vpnMode="wg"
        quicLabel={false}
        onSelect={onSelect}
        onNodeDetails={vi.fn()}
      />,
    );

    await user.click(screen.getByText('gateway one'));

    expect(onSelect).toHaveBeenCalledOnce();
  });

  it('does not call onSelect when the gateway is already selected for this hop', async () => {
    const onSelect = vi.fn();
    const user = userEvent.setup();
    renderWithProviders(
      <GatewayItem
        gateway={gateway({ isSelected: 'entry' })}
        node="entry"
        vpnMode="wg"
        quicLabel={false}
        onSelect={onSelect}
        onNodeDetails={vi.fn()}
      />,
    );

    await user.click(screen.getByText('gateway one'));

    expect(onSelect).not.toHaveBeenCalled();
  });

  it('opens node details via the chevron button', async () => {
    const onNodeDetails = vi.fn();
    const user = userEvent.setup();
    renderWithProviders(
      <GatewayItem
        gateway={gateway()}
        node="entry"
        vpnMode="wg"
        quicLabel={false}
        onSelect={vi.fn()}
        onNodeDetails={onNodeDetails}
      />,
    );

    await user.click(screen.getByTestId('button-icon'));

    expect(onNodeDetails).toHaveBeenCalledOnce();
  });
});
