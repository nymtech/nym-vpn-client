import React from 'react';
import { screen } from '@testing-library/react';
import { render } from '../../../test/test-utils';
import ConnectionBadge from '../ConnectionBadge';
import { TunnelState } from '../../../types';

jest.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        'status.connected': 'Connected',
        'status.disconnected': 'Disconnected',
        'status.connecting': 'Connecting',
        'status.disconnecting': 'Disconnecting',
        'status.error': 'Error',
        'status.offline': 'Offline',
      };
      return translations[key] || key;
    },
  }),
}));

describe('ConnectionBadge Component', () => {
  const tunnelStates: TunnelState[] = [
    'connected',
    'disconnected',
    'connecting',
    'disconnecting',
    'error',
    'offline',
    'offline-auto-reconnect',
    'unknown',
  ];

  describe('Basic Rendering', () => {
    it('renders the badge container correctly', () => {
      render(<ConnectionBadge state="disconnected" />);

      const badge = screen.getByTestId('connection-badge');
      expect(badge).toBeInTheDocument();
      expect(badge).toHaveAttribute('data-status', 'disconnected');
    });

    it('shows status text for all states', () => {
      tunnelStates.forEach((state) => {
        const { unmount } = render(<ConnectionBadge state={state} />);

        const badge = screen.getByTestId('connection-badge');
        const statusText = screen.getByTestId('connection-status-text');

        expect(badge).toHaveAttribute('data-status', state);
        expect(statusText).toBeInTheDocument();

        unmount();
      });
    });
  });

  describe('Status Text Display', () => {
    it('shows "Connected" for connected state', () => {
      render(<ConnectionBadge state="connected" />);

      expect(screen.getByTestId('connection-status-text')).toHaveTextContent(
        'Connected',
      );
    });

    it('shows "Disconnected" for disconnected state', () => {
      render(<ConnectionBadge state="disconnected" />);

      expect(screen.getByTestId('connection-status-text')).toHaveTextContent(
        'Disconnected',
      );
    });

    it('shows "Disconnected" for unknown state', () => {
      render(<ConnectionBadge state="unknown" />);

      expect(screen.getByTestId('connection-status-text')).toHaveTextContent(
        'Disconnected',
      );
    });

    it('shows "Connecting" for connecting state', () => {
      render(<ConnectionBadge state="connecting" />);

      expect(screen.getByTestId('connection-status-text')).toHaveTextContent(
        'Connecting',
      );
    });

    it('shows "Disconnecting" for disconnecting state', () => {
      render(<ConnectionBadge state="disconnecting" />);

      expect(screen.getByTestId('connection-status-text')).toHaveTextContent(
        'Disconnecting',
      );
    });

    it('shows "Error" for error state', () => {
      render(<ConnectionBadge state="error" />);

      expect(screen.getByTestId('connection-status-text')).toHaveTextContent(
        'Error',
      );
    });

    it('shows "Offline" for offline states', () => {
      const { unmount: unmount1 } = render(<ConnectionBadge state="offline" />);
      expect(screen.getByTestId('connection-status-text')).toHaveTextContent(
        'Offline',
      );
      unmount1();

      const { unmount: unmount2 } = render(
        <ConnectionBadge state="offline-auto-reconnect" />,
      );
      expect(screen.getByTestId('connection-status-text')).toHaveTextContent(
        'Offline',
      );
      unmount2();
    });
  });

  describe('Pulse Dot Display', () => {
    it('shows pulse dot for connecting state', () => {
      render(<ConnectionBadge state="connecting" />);

      const pulseDot = screen.getByTestId('connection-pulse-dot');
      expect(pulseDot).toBeInTheDocument();
      expect(pulseDot).toHaveAttribute('data-test-color', 'cornflower');
    });

    it('shows pulse dot for disconnecting state', () => {
      render(<ConnectionBadge state="disconnecting" />);

      const pulseDot = screen.getByTestId('connection-pulse-dot');
      expect(pulseDot).toBeInTheDocument();
      expect(pulseDot).toHaveAttribute('data-test-color', 'cornflower');
    });

    it('does not show pulse dot for stable states', () => {
      const stableStates: TunnelState[] = [
        'connected',
        'disconnected',
        'error',
        'offline',
        'offline-auto-reconnect',
        'unknown',
      ];

      stableStates.forEach((state) => {
        const { unmount } = render(<ConnectionBadge state={state} />);

        expect(
          screen.queryByTestId('connection-pulse-dot'),
        ).not.toBeInTheDocument();

        unmount();
      });
    });
  });
});
