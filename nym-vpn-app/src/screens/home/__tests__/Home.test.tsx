import React from 'react';
import { screen, render as rtlRender } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { invoke } from '@tauri-apps/api/core';
import Home from '../Home';
import {
  useMainDispatch,
  useMainState,
  useNodeListState,
} from '../../../contexts';

const mockNavigate = jest.fn();
const mockDispatch = jest.fn();
const mockInvoke = invoke as jest.MockedFunction<typeof invoke>;
const mockUseMainState = useMainState as jest.MockedFunction<
  typeof useMainState
>;
const mockUseMainDispatch = useMainDispatch as jest.MockedFunction<
  typeof useMainDispatch
>;

jest.mock('react-router', () => ({
  ...(jest.requireActual('react-router') as any),
  useNavigate: () => mockNavigate,
}));

jest.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        'connect-button': 'Connect',
        'disconnect-button': 'Disconnect',
        'select-node-title': 'Select nodes',
      };
      return translations[key] || key;
    },
  }),
}));

jest.mock('../TunnelState', () => {
  return function MockTunnelState() {
    return <div data-testid="tunnel-state">TunnelState</div>;
  };
});

jest.mock('../NetworkModeSelect', () => {
  return function MockNetworkModeSelect() {
    return <div data-testid="network-mode-select">NetworkModeSelect</div>;
  };
});

jest.mock('../HopSelect', () => {
  return function MockHopSelect({
    nodeHop,
    disabled,
    locked,
    onClick,
    node,
    gatewayId,
  }: any) {
    return (
      <div
        data-testid={`hop-select-${nodeHop}`}
        data-disabled={disabled}
        data-locked={locked}
        onClick={onClick}
      >
        HopSelect {nodeHop} - {node?.country || 'No country'} -{' '}
        {gatewayId || 'No gateway'}
      </div>
    );
  };
});

jest.mock('../NetworkUpdateDialog', () => {
  return function MockNetworkUpdateDialog({ isOpen, onClose }: any) {
    return isOpen ? (
      <div data-testid="network-update-dialog">NetworkUpdateDialog</div>
    ) : null;
  };
});

jest.mock('../UpdateDialog', () => {
  return function MockUpdateDialog() {
    return <div data-testid="update-dialog">UpdateDialog</div>;
  };
});

const render = (ui: React.ReactElement) => rtlRender(ui);

describe('Home Component', () => {
  beforeEach(() => {
    jest.clearAllMocks();

    (
      useNodeListState as jest.MockedFunction<typeof useNodeListState>
    ).mockReturnValue({
      reset: jest.fn(),
      entry: {
        expanded: [],
        focused: null,
        search: null,
      },
      exit: {
        expanded: [],
        focused: null,
        search: null,
      },
      setExpanded: jest.fn(),
      addToExpanded: jest.fn(),
      setFocused: jest.fn(),
      setSearch: jest.fn(),
    });

    Object.defineProperty(window, '_APP', {
      value: {
        updaterEnabled: false,
        devMode: false,
      },
      writable: true,
    });

    mockUseMainDispatch.mockReturnValue(mockDispatch);

    mockUseMainState.mockReturnValue({
      state: 'disconnected',
      tunnel: null,
      connectingState: null,
      accountState: 'active',
      entryNode: null,
      exitNode: null,
      daemonStatus: 'up',
      account: { id: 'test-account' },
      networkCompat: { tauri: true, core: true },
      welcomeChecked: true,
    } as any);
  });

  describe('Basic Rendering', () => {
    it('renders main container and components', () => {
      render(<Home />);

      expect(screen.getByTestId('home-container')).toBeInTheDocument();
      expect(
        screen.getByTestId('home-tunnel-state-container'),
      ).toBeInTheDocument();
      expect(screen.getByTestId('home-controls-container')).toBeInTheDocument();
      expect(screen.getByTestId('tunnel-state')).toBeInTheDocument();
      expect(screen.getByTestId('network-mode-select')).toBeInTheDocument();
    });

    it('shows node selection section', () => {
      render(<Home />);

      expect(
        screen.getByTestId('home-node-select-section'),
      ).toBeInTheDocument();
      expect(screen.getByTestId('home-node-select-title')).toHaveTextContent(
        'Select nodes',
      );
      expect(
        screen.getByTestId('home-hop-selects-container'),
      ).toBeInTheDocument();
    });

    it('renders hop selects for entry and exit nodes', () => {
      render(<Home />);

      expect(screen.getByTestId('hop-select-entry')).toBeInTheDocument();
      expect(screen.getByTestId('hop-select-exit')).toBeInTheDocument();
    });

    it('shows connection button', () => {
      render(<Home />);

      const button = screen.getByTestId('home-connection-button');
      expect(button).toBeInTheDocument();
    });
  });

  describe('Connection Button States', () => {
    it('shows connect button when disconnected', () => {
      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        daemonStatus: 'up',
        account: { id: 'test-account' },
      } as any);

      render(<Home />);

      const button = screen.getByTestId('home-connection-button');
      expect(button).not.toBeDisabled();
      expect(button).toHaveTextContent('connect');
    });

    it('shows disconnect button when connected', () => {
      mockUseMainState.mockReturnValue({
        state: 'connected',
        daemonStatus: 'up',
        account: { id: 'test-account' },
      } as any);

      render(<Home />);

      const button = screen.getByTestId('home-connection-button');
      expect(button).not.toBeDisabled();
      expect(button).toHaveTextContent('disconnect');
    });

    it('disables button when daemon is down', () => {
      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        daemonStatus: 'down',
        account: { id: 'test-account' },
      } as any);

      render(<Home />);

      const button = screen.getByTestId('home-connection-button');
      expect(button).toBeDisabled();
    });

    it('disables button when offline', () => {
      mockUseMainState.mockReturnValue({
        state: 'offline',
        daemonStatus: 'up',
        account: { id: 'test-account' },
      } as any);

      render(<Home />);

      const button = screen.getByTestId('home-connection-button');
      expect(button).toBeDisabled();
    });

    it('shows loading state when disconnecting', () => {
      mockUseMainState.mockReturnValue({
        state: 'disconnecting',
        daemonStatus: 'up',
        account: { id: 'test-account' },
      } as any);

      render(<Home />);

      const button = screen.getByTestId('home-connection-button');
      expect(button).toBeDisabled();
      expect(screen.getByTestId('button-spinner')).toBeInTheDocument();
    });
  });

  describe('Hop Select Behavior', () => {
    it('enables hop selects when disconnected and daemon is up', () => {
      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        daemonStatus: 'up',
        account: { id: 'test-account' },
      } as any);

      render(<Home />);

      expect(screen.getByTestId('hop-select-entry')).toHaveAttribute(
        'data-disabled',
        'false',
      );
      expect(screen.getByTestId('hop-select-exit')).toHaveAttribute(
        'data-disabled',
        'false',
      );
    });

    it('disables hop selects when connected', () => {
      mockUseMainState.mockReturnValue({
        state: 'connected',
        daemonStatus: 'up',
        account: { id: 'test-account' },
      } as any);

      render(<Home />);

      expect(screen.getByTestId('hop-select-entry')).toHaveAttribute(
        'data-disabled',
        'true',
      );
      expect(screen.getByTestId('hop-select-exit')).toHaveAttribute(
        'data-disabled',
        'true',
      );
    });

    it('locks hop selects when daemon is down', () => {
      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        daemonStatus: 'down',
        account: { id: 'test-account' },
      } as any);

      render(<Home />);

      expect(screen.getByTestId('hop-select-entry')).toHaveAttribute(
        'data-locked',
        'true',
      );
      expect(screen.getByTestId('hop-select-exit')).toHaveAttribute(
        'data-locked',
        'true',
      );
    });

    it('navigates to node selection when hop select is clicked', async () => {
      const user = userEvent.setup();

      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        daemonStatus: 'up',
        account: { id: 'test-account' },
      } as any);

      render(<Home />);

      await user.click(screen.getByTestId('hop-select-entry'));
      expect(mockNavigate).toHaveBeenCalledWith('/entry-node');

      await user.click(screen.getByTestId('hop-select-exit'));
      expect(mockNavigate).toHaveBeenCalledWith('/exit-node');
    });
  });

  describe('Connection Logic', () => {
    it('redirects to login when not authenticated', async () => {
      const user = userEvent.setup();

      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        daemonStatus: 'up',
        account: null, // No account
      } as any);

      render(<Home />);

      const button = screen.getByTestId('home-connection-button');
      await user.click(button);

      expect(mockNavigate).toHaveBeenCalledTimes(1);
    });

    it('attempts to connect when authenticated and disconnected', async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue(undefined);

      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        daemonStatus: 'up',
        account: { id: 'test-account' },
      } as any);

      render(<Home />);

      const button = screen.getByTestId('home-connection-button');
      await user.click(button);

      expect(mockInvoke).toHaveBeenCalledWith('connect', {
        entry: undefined,
        exit: undefined,
      });
    });

    it('attempts to disconnect when connected', async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue(undefined);

      mockUseMainState.mockReturnValue({
        state: 'connected',
        daemonStatus: 'up',
        account: { id: 'test-account' },
      } as any);

      render(<Home />);

      const button = screen.getByTestId('home-connection-button');
      await user.click(button);

      expect(mockInvoke).toHaveBeenCalledWith('disconnect');
    });
  });

  describe('Account States', () => {
    it('shows different button behavior for no subscription', () => {
      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        daemonStatus: 'up',
        account: { id: 'test-account' },
        accountState: 'no-subscription',
      } as any);

      render(<Home />);

      const button = screen.getByTestId('home-connection-button');
      expect(button).toHaveTextContent('get-started');
    });

    it('shows different button behavior for bandwidth exceeded', () => {
      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        daemonStatus: 'up',
        account: { id: 'test-account' },
        accountState: 'bandwidth-exceeded',
      } as any);

      render(<Home />);

      const button = screen.getByTestId('home-connection-button');
      expect(button).toHaveTextContent('get-started');
    });
  });

  describe('Node Information Display', () => {
    it('displays node information when available', () => {
      const entryNode = { country: 'Germany', location: 'Berlin' };
      const exitNode = { country: 'Sweden', location: 'Stockholm' };

      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        daemonStatus: 'up',
        account: { id: 'test-account' },
        entryNode,
        exitNode,
      } as any);

      render(<Home />);

      expect(screen.getByTestId('hop-select-entry')).toHaveTextContent(
        'Germany',
      );
      expect(screen.getByTestId('hop-select-exit')).toHaveTextContent('Sweden');
    });

    it('shows gateway IDs when connected', () => {
      const tunnel = { entryGwId: 'entry-123', exitGwId: 'exit-456' };

      mockUseMainState.mockReturnValue({
        state: 'connected',
        daemonStatus: 'up',
        account: { id: 'test-account' },
        tunnel,
      } as any);

      render(<Home />);

      expect(screen.getByTestId('hop-select-entry')).toHaveTextContent(
        'entry-123',
      );
      expect(screen.getByTestId('hop-select-exit')).toHaveTextContent(
        'exit-456',
      );
    });
  });
});
