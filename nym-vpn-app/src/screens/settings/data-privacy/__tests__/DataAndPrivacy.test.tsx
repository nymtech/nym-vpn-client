import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { render } from '../../../../test/test-utils';
import DataAndPrivacy from '../DataAndPrivacy';
import { useMainState, useMainDispatch } from '../../../../contexts';

const mockInvoke = invoke as jest.MockedFunction<typeof invoke>;
const mockUseMainState = useMainState as jest.MockedFunction<
  typeof useMainState
>;
const mockUseMainDispatch = useMainDispatch as jest.MockedFunction<
  typeof useMainDispatch
>;

const mockDispatch = jest.fn();

describe('DataAndPrivacy Component', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockUseMainDispatch.mockReturnValue(mockDispatch);
  });

  describe('Rendering', () => {
    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        monitoring: false,
        networkStats: false,
      } as any);
    });

    it('renders both privacy settings cards', () => {
      render(<DataAndPrivacy />);

      expect(
        screen.getByText('privacy.network stats.label'),
      ).toBeInTheDocument();
      expect(
        screen.getByText('privacy.error monitoring.label'),
      ).toBeInTheDocument();
    });

    it('renders network stats card with correct content', () => {
      render(<DataAndPrivacy />);

      expect(
        screen.getByText('privacy.network stats.sublabel'),
      ).toBeInTheDocument();
      expect(
        screen.getByText('privacy.network stats.desc'),
      ).toBeInTheDocument();
      expect(
        screen.getByText('privacy.network stats.link'),
      ).toBeInTheDocument();
    });

    it('renders error monitoring card with correct content', () => {
      render(<DataAndPrivacy />);

      expect(
        screen.getByText('privacy.error monitoring.sublabel'),
      ).toBeInTheDocument();
      expect(
        screen.getByText('privacy.error monitoring.desc'),
      ).toBeInTheDocument();
      expect(
        screen.getByText('privacy.error monitoring.link'),
      ).toBeInTheDocument();
    });

    it('shows switches in unchecked state by default', () => {
      render(<DataAndPrivacy />);

      const switches = screen.getAllByTestId('switch');
      const networkStatsSwitch = switches[0];
      const errorMonitoringSwitch = switches[1];

      expect(networkStatsSwitch).not.toHaveAttribute('data-checked', 'true');
      expect(errorMonitoringSwitch).not.toHaveAttribute('data-checked', 'true');
    });
  });

  describe('Network Stats Toggle', () => {
    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        monitoring: false,
        networkStats: false,
      } as any);
    });

    it('toggles network stats when clicked', async () => {
      render(<DataAndPrivacy />);

      const switches = screen.getAllByTestId('switch');
      const networkStatsSwitch = switches[0];
      fireEvent.click(networkStatsSwitch);

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'set-network-stats',
          enabled: true,
        });
      });
    });

    it('calls enable_netstats when enabling', async () => {
      render(<DataAndPrivacy />);

      const switches = screen.getAllByTestId('switch');
      const networkStatsSwitch = switches[0];
      fireEvent.click(networkStatsSwitch);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('enable_netstats');
      });
    });

    it('calls disable_netstats when disabling', async () => {
      mockUseMainState.mockReturnValue({
        monitoring: false,
        networkStats: true,
      } as any);

      render(<DataAndPrivacy />);

      const switches = screen.getAllByTestId('switch');
      const networkStatsSwitch = switches[0];
      fireEvent.click(networkStatsSwitch);

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'set-network-stats',
          enabled: false,
        });
        expect(mockInvoke).toHaveBeenCalledWith('disable_netstats');
      });
    });

    it('handles network stats toggle errors gracefully', async () => {
      mockInvoke.mockRejectedValue(new Error('API error'));

      render(<DataAndPrivacy />);

      const switches = screen.getAllByTestId('switch');
      const networkStatsSwitch = switches[0];
      fireEvent.click(networkStatsSwitch);

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'set-network-stats',
          enabled: true,
        });
        expect(mockInvoke).toHaveBeenCalledWith('enable_netstats');
      });
    });
  });

  describe('Error Monitoring Toggle', () => {
    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        monitoring: false,
        networkStats: false,
      } as any);
    });

    it('toggles error monitoring when clicked', async () => {
      render(<DataAndPrivacy />);

      const errorMonitoringSwitch = screen.getAllByTestId('switch')[1];
      fireEvent.click(errorMonitoringSwitch);

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'set-monitoring',
          enabled: true,
        });
      });
    });

    it('calls enable_sentry when enabling', async () => {
      render(<DataAndPrivacy />);

      const errorMonitoringSwitch = screen.getAllByTestId('switch')[1];
      fireEvent.click(errorMonitoringSwitch);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('enable_sentry');
      });
    });

    it('calls disable_sentry when disabling', async () => {
      mockUseMainState.mockReturnValue({
        monitoring: true,
        networkStats: false,
      } as any);

      render(<DataAndPrivacy />);

      const errorMonitoringSwitch = screen.getAllByTestId('switch')[1];
      fireEvent.click(errorMonitoringSwitch);

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'set-monitoring',
          enabled: false,
        });
        expect(mockInvoke).toHaveBeenCalledWith('disable_sentry');
      });
    });

    it('handles error monitoring toggle errors gracefully', async () => {
      mockInvoke.mockRejectedValue(new Error('API error'));

      render(<DataAndPrivacy />);

      const errorMonitoringSwitch = screen.getAllByTestId('switch')[1];
      fireEvent.click(errorMonitoringSwitch);

      await waitFor(() => {
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'set-monitoring',
          enabled: true,
        });
        expect(mockInvoke).toHaveBeenCalledWith('enable_sentry');
      });
    });
  });

  describe('Link Functionality', () => {
    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        monitoring: false,
        networkStats: false,
      } as any);
    });

    it('renders network stats link', () => {
      render(<DataAndPrivacy />);

      const networkStatsLink = screen.getByText('privacy.network stats.link');
      expect(networkStatsLink).toBeInTheDocument();
    });

    it('renders error monitoring link', () => {
      render(<DataAndPrivacy />);

      const errorMonitoringLink = screen.getByText(
        'privacy.error monitoring.link',
      );
      expect(errorMonitoringLink).toBeInTheDocument();
    });
  });

  describe('State Management', () => {
    it('reflects current state in switches', () => {
      mockUseMainState.mockReturnValue({
        monitoring: true,
        networkStats: false,
      } as any);

      render(<DataAndPrivacy />);

      const switches = screen.getAllByTestId('switch');
      const networkStatsSwitch = switches[0];
      const errorMonitoringSwitch = switches[1];

      expect(networkStatsSwitch).toBeInTheDocument();
      expect(errorMonitoringSwitch).toBeInTheDocument();
    });
  });
});
