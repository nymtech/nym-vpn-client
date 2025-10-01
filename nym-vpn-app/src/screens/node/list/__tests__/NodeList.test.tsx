import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { userEvent } from '@testing-library/user-event';
import NodeList from '../NodeList';
import {
  UiGatewaysByCountry,
  UiGateway,
  UiCountry,
  SelectedKind,
} from '../../../../contexts';
import { NodeHop, VpnMode } from '../../../../types';

jest.mock('../CountryInfo', () => {
  return function MockCountryInfo({
    country,
    name,
    gwCount,
  }: {
    country: UiCountry;
    name: string;
    gwCount: number;
  }) {
    return (
      <div data-testid={`mocked-country-info-${country.code}`}>
        {name} ({gwCount})
      </div>
    );
  };
});

jest.mock('../GatewayItem', () => {
  return function MockGatewayItem({
    gateway,
    node,
    onSelect,
    onNodeDetails,
  }: {
    gateway: UiGateway;
    node: NodeHop;
    onSelect: (gw: UiGateway) => void;
    onNodeDetails: (gw: UiGateway) => void;
  }) {
    return (
      <div data-testid={`mocked-gateway-item-${gateway.id.substring(0, 8)}`}>
        <button onClick={() => onSelect(gateway)}>Select {gateway.name}</button>
        <button onClick={() => onNodeDetails(gateway)}>
          Info {gateway.name}
        </button>
      </div>
    );
  };
});

jest.mock('../FoldButton', () => {
  return function MockFoldButton() {
    return <div data-testid="mocked-fold-button">Fold</div>;
  };
});

describe('NodeList', () => {
  const mockCountryUS: UiCountry = {
    code: 'US',
    name: 'United States',
    isSelected: false,
  };

  const mockCountryDE: UiCountry = {
    code: 'DE',
    name: 'Germany',
    isSelected: false,
  };

  const createMockGateway = (
    id: string,
    name: string,
    overrides: Partial<UiGateway> = {},
  ): UiGateway => ({
    id,
    name,
    country: mockCountryUS,
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

  const mockNodes: UiGatewaysByCountry[] = [
    {
      i18n: 'United States',
      isSelected: false,
      country: mockCountryUS,
      type: 'wg' as const,
      gateways: [
        createMockGateway('us-gateway-1', 'US Gateway 1'),
        createMockGateway('us-gateway-2', 'US Gateway 2'),
      ],
    },
    {
      i18n: 'Germany',
      isSelected: 'entry' as SelectedKind,
      country: mockCountryDE,
      type: 'wg' as const,
      gateways: [
        createMockGateway('de-gateway-1', 'DE Gateway 1', {
          country: mockCountryDE,
        }),
      ],
    },
  ];

  const mockStandaloneGateways: UiGateway[] = [
    createMockGateway('standalone-1', 'Standalone Gateway 1'),
    createMockGateway('standalone-2', 'Standalone Gateway 2'),
  ];

  const mockProps = {
    nodes: mockNodes,
    gateways: [] as UiGateway[],
    onSelect: jest.fn(),
    onNodeDetails: jest.fn(),
    node: 'entry' as NodeHop,
    vpnMode: 'wg' as VpnMode,
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Rendering', () => {
    it('renders accordion root with correct test ID', () => {
      render(<NodeList {...mockProps} />);

      expect(screen.getByTestId('node-list-accordion')).toBeInTheDocument();
    });

    it('renders country accordion items', () => {
      render(<NodeList {...mockProps} />);

      expect(
        screen.getByTestId('country-accordion-item-US'),
      ).toBeInTheDocument();
      expect(
        screen.getByTestId('country-accordion-item-DE'),
      ).toBeInTheDocument();

      expect(screen.getByTestId('country-header-US')).toBeInTheDocument();
      expect(screen.getByTestId('country-header-DE')).toBeInTheDocument();
    });

    it('renders country info components', () => {
      render(<NodeList {...mockProps} />);

      expect(screen.getByTestId('mocked-country-info-US')).toHaveTextContent(
        'United States (2)',
      );
      expect(screen.getByTestId('mocked-country-info-DE')).toHaveTextContent(
        'Germany (1)',
      );
    });

    it('renders selection indicators with correct states', () => {
      render(<NodeList {...mockProps} />);

      const usIndicator = screen.getByTestId('country-selection-indicator-US');
      const deIndicator = screen.getByTestId('country-selection-indicator-DE');

      expect(usIndicator).toHaveAttribute('data-selected', 'none');
      expect(deIndicator).toHaveAttribute('data-selected', 'entry');
    });

    it('renders fold buttons in accordion headers', () => {
      render(<NodeList {...mockProps} />);

      expect(
        screen.getByTestId('country-accordion-header-US'),
      ).toBeInTheDocument();
      expect(
        screen.getByTestId('country-accordion-header-DE'),
      ).toBeInTheDocument();
      expect(screen.getAllByTestId('mocked-fold-button')).toHaveLength(2);
    });

    it('renders accordion content containers', () => {
      render(<NodeList {...mockProps} />);

      expect(
        screen.getByTestId('country-accordion-content-US'),
      ).toBeInTheDocument();
      expect(
        screen.getByTestId('country-accordion-content-DE'),
      ).toBeInTheDocument();
    });

    it('renders standalone gateways section', () => {
      render(<NodeList {...mockProps} gateways={mockStandaloneGateways} />);

      expect(
        screen.getByTestId('standalone-gateways-container'),
      ).toBeInTheDocument();
      expect(screen.getAllByTestId(/standalone-gateway-standalo/)).toHaveLength(
        2,
      );
      expect(
        screen.getAllByTestId(/mocked-gateway-item-standalo/),
      ).toHaveLength(2);
    });

    it('does not render standalone gateways when none exist', () => {
      render(<NodeList {...mockProps} gateways={[]} />);

      expect(
        screen.getByTestId('standalone-gateways-container'),
      ).toBeInTheDocument();
      expect(
        screen.queryByTestId(/standalone-gateway-/),
      ).not.toBeInTheDocument();
    });
  });

  describe('Country Selection Logic', () => {
    it('calls onSelect when clicking unselected country', async () => {
      const user = userEvent.setup();
      render(<NodeList {...mockProps} />);

      const selectArea = screen.getByTestId('country-select-area-US');
      await user.click(selectArea);

      expect(mockProps.onSelect).toHaveBeenCalledWith(mockCountryUS);
    });

    it('does not call onSelect when clicking country selected by current hop', async () => {
      const user = userEvent.setup();
      render(<NodeList {...mockProps} />);

      const selectArea = screen.getByTestId('country-select-area-DE');
      await user.click(selectArea);

      expect(mockProps.onSelect).not.toHaveBeenCalled();
    });

    it('does not call onSelect when clicking country selected by other hop with single gateway', async () => {
      const user = userEvent.setup();
      const nodesWithExitSelection = [
        {
          i18n: 'United States',
          isSelected: 'exit' as SelectedKind,
          country: mockCountryUS,
          type: 'wg' as const,
          gateways: [createMockGateway('us-gateway-1', 'US Gateway 1')],
        },
      ];

      render(
        <NodeList {...mockProps} nodes={nodesWithExitSelection} node="entry" />,
      );

      const selectArea = screen.getByTestId('country-select-area-US');
      await user.click(selectArea);

      expect(mockProps.onSelect).not.toHaveBeenCalled();
    });

    it('calls onSelect when clicking country selected by other hop with multiple gateways', async () => {
      const user = userEvent.setup();
      const nodesWithExitSelection = [
        {
          i18n: 'United States',
          isSelected: 'exit' as SelectedKind,
          country: mockCountryUS,
          type: 'wg' as const,
          gateways: [
            createMockGateway('us-gateway-1', 'US Gateway 1'),
            createMockGateway('us-gateway-2', 'US Gateway 2'),
          ],
        },
      ];

      render(
        <NodeList {...mockProps} nodes={nodesWithExitSelection} node="entry" />,
      );

      const selectArea = screen.getByTestId('country-select-area-US');
      await user.click(selectArea);

      expect(mockProps.onSelect).toHaveBeenCalledWith(mockCountryUS);
    });

    it('does not call onSelect when clicking entry-and-exit selected country', async () => {
      const user = userEvent.setup();
      const nodesWithBothSelection = [
        {
          i18n: 'United States',
          isSelected: 'entry-and-exit' as SelectedKind,
          country: mockCountryUS,
          type: 'wg' as const,
          gateways: [createMockGateway('us-gateway-1', 'US Gateway 1')],
        },
      ];

      render(<NodeList {...mockProps} nodes={nodesWithBothSelection} />);

      const selectArea = screen.getByTestId('country-select-area-US');
      await user.click(selectArea);

      expect(mockProps.onSelect).not.toHaveBeenCalled();
    });
  });

  describe('Edge Cases', () => {
    it('handles empty nodes array', () => {
      render(<NodeList {...mockProps} nodes={[]} />);

      expect(screen.getByTestId('node-list-accordion')).toBeInTheDocument();
      expect(
        screen.queryByTestId(/country-accordion-item-/),
      ).not.toBeInTheDocument();
    });

    it('handles nodes with empty gateways arrays', () => {
      const nodesWithoutGateways = [
        {
          i18n: 'Empty Country',
          isSelected: false as const,
          country: {
            code: 'XX',
            name: 'Empty Country',
            isSelected: false as const,
          },
          type: 'wg' as const,
          gateways: [],
        },
      ];

      render(<NodeList {...mockProps} nodes={nodesWithoutGateways} />);

      expect(
        screen.getByTestId('country-accordion-item-XX'),
      ).toBeInTheDocument();
      expect(screen.getByTestId('mocked-country-info-XX')).toHaveTextContent(
        'Empty Country (0)',
      );
    });
  });
});
