import React from 'react';
import { render } from '@testing-library/react';
import { UiGateway, UiCountry } from '../../../../contexts';
import { NodeHop, VpnMode } from '../../../../types';
import GatewayItem from '../GatewayItem';

describe('GatewayItem', () => {
  const mockCountry: UiCountry = {
    code: 'US',
    name: 'United States',
    isSelected: false,
  };

  const createMockGateway = (
    overrides: Partial<UiGateway> = {},
  ): UiGateway => ({
    id: 'test-gateway-12345',
    name: 'Test Gateway',
    country: mockCountry,
    type: 'wg' as const,
    location: {
      latitude: 40.7128,
      longitude: -74.006,
      city: 'New York',
      region: 'NY',
    },
    asn: { asn: '12345', name: 'Test ASN', type: 'other' as const },
    wgScore: 'high' as const,
    mxScore: 'medium' as const,
    wgPerformance: null,
    exitIpv4: '192.168.1.1',
    exitIpv6: null,
    buildVersion: '1.0.0',
    isSelected: false,
    ...overrides,
  });

  const mockProps = {
    gateway: createMockGateway(),
    node: 'entry' as NodeHop,
    vpnMode: 'wg' as VpnMode,
    onSelect: jest.fn(),
    onNodeDetails: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders basic gateway structure', () => {
      const { container } = render(<GatewayItem {...mockProps} />);
      expect(container.firstChild).toBeInTheDocument();
    });

    it('renders with different gateway data', () => {
      const gateway = createMockGateway({
        name: 'Custom Gateway',
        id: 'custom-id-123',
      });

      const { container } = render(
        <GatewayItem {...mockProps} gateway={gateway} />,
      );
      expect(container.firstChild).toBeInTheDocument();
    });
  });

  describe('Props Handling', () => {
    it('accepts different node hop types', () => {
      const { container } = render(<GatewayItem {...mockProps} node="exit" />);
      expect(container.firstChild).toBeInTheDocument();
    });

    it('accepts different VPN modes', () => {
      const { container } = render(
        <GatewayItem {...mockProps} vpnMode="mixnet" />,
      );
      expect(container.firstChild).toBeInTheDocument();
    });
  });
});
