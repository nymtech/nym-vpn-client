import React from 'react';
import { screen, waitFor } from '@testing-library/react';
import { render } from '../../../test/test-utils';
import TunnelState from '../TunnelState';
import { useMainState } from '../../../contexts';
import {
  useI18nAccountState,
  useI18nError,
  useI18nProgressMsg,
} from '../../../hooks';

const mockUseMainState = useMainState as jest.MockedFunction<
  typeof useMainState
>;
const mockUseI18nError = useI18nError as jest.MockedFunction<
  typeof useI18nError
>;
const mockUseI18nAccountState = useI18nAccountState as jest.MockedFunction<
  typeof useI18nAccountState
>;
const mockUseI18nProgressMsg = useI18nProgressMsg as jest.MockedFunction<
  typeof useI18nProgressMsg
>;

jest.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: any) => {
      const translations: Record<string, string> = {
        'offline-message': 'You are offline',
        'offline-reconnect-message': 'Reconnecting...',
        'connection-attempt': `Connection attempt ${options?.count || 1}`,
      };
      return translations[key] || key;
    },
  }),
}));

jest.mock('../ConnectionBadge', () => {
  return function MockConnectionBadge({ state }: { state: string }) {
    return (
      <div data-testid="connection-badge" data-status={state}>
        Badge: {state}
      </div>
    );
  };
});

jest.mock('../ConnectionTimer', () => {
  return function MockConnectionTimer() {
    return <div data-testid="connection-timer">Timer</div>;
  };
});

describe('TunnelState Component', () => {
  beforeEach(() => {
    jest.clearAllMocks();

    mockUseI18nError.mockReturnValue({
      tE: (error: any) => {
        if (typeof error === 'string') {
          return `Error: ${error}`;
        }
        if (error && typeof error === 'object' && error.key) {
          return `Error: ${error.key}`;
        }
        return 'Error: unknown';
      },
    });

    mockUseI18nAccountState.mockReturnValue({
      t: (state: string) => `Account: ${state}`,
    });

    mockUseI18nProgressMsg.mockReturnValue({
      t: (msg: string) => `Progress: ${msg}`,
    });
  });

  describe('Basic Rendering', () => {
    it('renders the main container correctly', () => {
      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        error: null,
        progressMessages: [],
        tunnelError: null,
        connectingState: null,
        accountState: 'active',
        accountError: null,
      } as any);

      render(<TunnelState />);

      expect(screen.getByTestId('tunnel-state-container')).toBeInTheDocument();
      expect(screen.getByTestId('tunnel-badge-container')).toBeInTheDocument();
      expect(
        screen.getByTestId('tunnel-details-container'),
      ).toBeInTheDocument();
    });

    it('shows connection badge for all states', async () => {
      mockUseMainState.mockReturnValue({
        state: 'connected',
        error: null,
        progressMessages: [],
        tunnelError: null,
        connectingState: null,
        accountState: 'active',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(screen.getByTestId('connection-badge')).toBeInTheDocument();
        expect(screen.getByTestId('connection-badge')).toHaveAttribute(
          'data-status',
          'connected',
        );
      });
    });
  });

  describe('Connected State', () => {
    it('shows connection timer when connected', async () => {
      mockUseMainState.mockReturnValue({
        state: 'connected',
        error: null,
        progressMessages: [],
        tunnelError: null,
        connectingState: null,
        accountState: 'active',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(screen.getByTestId('connection-timer')).toBeInTheDocument();
      });
    });

    it('does not show error when connected without issues', async () => {
      mockUseMainState.mockReturnValue({
        state: 'connected',
        error: null,
        progressMessages: [],
        tunnelError: null,
        connectingState: null,
        accountState: 'active',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(
          screen.queryByTestId('tunnel-error-container'),
        ).not.toBeInTheDocument();
        expect(
          screen.queryByTestId('tunnel-info-message'),
        ).not.toBeInTheDocument();
      });
    });
  });

  describe('Connecting State', () => {
    it('shows progress messages during connection', async () => {
      mockUseMainState.mockReturnValue({
        state: 'connecting',
        error: null,
        progressMessages: ['Establishing connection'],
        tunnelError: null,
        connectingState: { progress: 'connecting-to-gateway' },
        accountState: 'active',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(screen.getByTestId('tunnel-info-message')).toBeInTheDocument();
        expect(
          screen.getByText('Progress: Establishing connection'),
        ).toBeInTheDocument();
      });
    });

    it('shows retry attempt count', async () => {
      mockUseMainState.mockReturnValue({
        state: 'connecting',
        error: null,
        progressMessages: [],
        tunnelError: null,
        connectingState: { retryAttempt: 3 },
        accountState: 'active',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(screen.getByTestId('tunnel-info-message')).toBeInTheDocument();
        expect(screen.getByText('Connection attempt 3')).toBeInTheDocument();
      });
    });

    it('shows connecting progress when available', async () => {
      mockUseMainState.mockReturnValue({
        state: 'connecting',
        error: null,
        progressMessages: [],
        tunnelError: null,
        connectingState: { progress: 'establishing-tunnel' },
        accountState: 'active',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(screen.getByTestId('tunnel-info-message')).toBeInTheDocument();
        expect(
          screen.getByText('Progress: establishing-tunnel'),
        ).toBeInTheDocument();
      });
    });
  });

  describe('Offline States', () => {
    it('shows offline message when offline', async () => {
      mockUseMainState.mockReturnValue({
        state: 'offline',
        error: null,
        progressMessages: [],
        tunnelError: null,
        connectingState: null,
        accountState: 'active',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(screen.getByTestId('tunnel-info-message')).toBeInTheDocument();
        expect(screen.getByText('You are offline')).toBeInTheDocument();
      });
    });

    it('shows reconnect message when offline auto-reconnect', async () => {
      mockUseMainState.mockReturnValue({
        state: 'offline-auto-reconnect',
        error: null,
        progressMessages: [],
        tunnelError: null,
        connectingState: null,
        accountState: 'active',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(screen.getByTestId('tunnel-info-message')).toBeInTheDocument();
        expect(screen.getByText('Reconnecting...')).toBeInTheDocument();
      });
    });
  });

  describe('Error Handling', () => {
    it('shows tunnel error when present', async () => {
      mockUseMainState.mockReturnValue({
        state: 'error',
        error: null,
        progressMessages: [],
        tunnelError: 'TUNNEL_FAILED',
        connectingState: null,
        accountState: 'active',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(
          screen.getByTestId('tunnel-error-container'),
        ).toBeInTheDocument();
        expect(screen.getByTestId('tunnel-specific-error')).toBeInTheDocument();
        expect(screen.getByText('Error: TUNNEL_FAILED')).toBeInTheDocument();
      });
    });

    it('shows account error when present', async () => {
      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        error: null,
        progressMessages: [],
        tunnelError: null,
        connectingState: null,
        accountState: 'no-subscription',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(
          screen.getByTestId('tunnel-error-container'),
        ).toBeInTheDocument();
        expect(
          screen.getByTestId('account-specific-error'),
        ).toBeInTheDocument();
        expect(
          screen.getByText('Account: no-subscription'),
        ).toBeInTheDocument();
      });
    });

    it('shows general error with details', async () => {
      const errorWithData = {
        key: 'NETWORK_ERROR',
        message: 'Network failed',
        data: { code: 500, reason: 'Server error' },
      };

      mockUseMainState.mockReturnValue({
        state: 'error',
        error: errorWithData,
        progressMessages: [],
        tunnelError: null,
        connectingState: null,
        accountState: 'active',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(
          screen.getByTestId('tunnel-error-container'),
        ).toBeInTheDocument();
        expect(screen.getByTestId('tunnel-error-key')).toBeInTheDocument();
        expect(screen.getByTestId('tunnel-error-data')).toBeInTheDocument();
        expect(screen.getByText('Error: NETWORK_ERROR')).toBeInTheDocument();
        expect(
          screen.getByText('{"code":500,"reason":"Server error"}'),
        ).toBeInTheDocument();
      });
    });

    it('shows error message when no key is provided', async () => {
      const errorWithoutKey = {
        message: 'Something went wrong',
        data: null,
      };

      mockUseMainState.mockReturnValue({
        state: 'error',
        error: errorWithoutKey,
        progressMessages: [],
        tunnelError: null,
        connectingState: null,
        accountState: 'active',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(
          screen.getByTestId('tunnel-error-container'),
        ).toBeInTheDocument();
        expect(screen.getByTestId('tunnel-error-key')).toBeInTheDocument();
        expect(screen.getByText('Something went wrong')).toBeInTheDocument();
      });
    });

    it('prioritizes tunnel error over account error', async () => {
      mockUseMainState.mockReturnValue({
        state: 'error',
        error: null,
        progressMessages: [],
        tunnelError: 'TUNNEL_FAILED',
        connectingState: null,
        accountState: 'no-subscription',
        accountError: null,
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(screen.getByTestId('tunnel-specific-error')).toBeInTheDocument();
        expect(
          screen.queryByTestId('account-specific-error'),
        ).not.toBeInTheDocument();
        expect(screen.getByText('Error: TUNNEL_FAILED')).toBeInTheDocument();
      });
    });
  });

  describe('Account Error States', () => {
    const accountErrorStates = [
      'max-device-reached',
      'no-subscription',
      'bandwidth-exceeded',
      'status-not-active',
      'error',
    ];

    accountErrorStates.forEach((accountState) => {
      it(`shows account error for ${accountState} state`, async () => {
        mockUseMainState.mockReturnValue({
          state: 'disconnected',
          error: null,
          progressMessages: [],
          tunnelError: null,
          connectingState: null,
          accountState,
          accountError: null,
        } as any);

        render(<TunnelState />);

        await waitFor(() => {
          expect(
            screen.getByTestId('tunnel-error-container'),
          ).toBeInTheDocument();
          expect(
            screen.getByTestId('account-specific-error'),
          ).toBeInTheDocument();
          expect(
            screen.getByText(`Account: ${accountState}`),
          ).toBeInTheDocument();
        });
      });
    });

    it('shows account error message when accountError is present', async () => {
      mockUseMainState.mockReturnValue({
        state: 'disconnected',
        error: null,
        progressMessages: [],
        tunnelError: null,
        connectingState: null,
        accountState: 'error',
        accountError: { key: 'ACCOUNT_SUSPENDED' },
      } as any);

      render(<TunnelState />);

      await waitFor(() => {
        expect(
          screen.getByTestId('tunnel-error-container'),
        ).toBeInTheDocument();
        expect(
          screen.getByTestId('account-specific-error'),
        ).toBeInTheDocument();
        expect(
          screen.getByText('Error: ACCOUNT_SUSPENDED'),
        ).toBeInTheDocument();
      });
    });
  });
});
