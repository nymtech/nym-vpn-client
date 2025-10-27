import React from 'react';
import { screen, fireEvent } from '@testing-library/react';
import { render } from '../../../../test/test-utils';
import InfoData from '../InfoData';
import { useMainState } from '../../../../contexts';
import { useClipboard } from '../../../../hooks';

const mockUseMainState = useMainState as jest.MockedFunction<
  typeof useMainState
>;
const mockUseClipboard = useClipboard as jest.MockedFunction<
  typeof useClipboard
>;

const mockCopy = jest.fn() as jest.MockedFunction<
  (text: string, notify?: boolean) => Promise<void>
>;

describe('InfoData Component', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockUseClipboard.mockReturnValue({ copy: mockCopy });
  });

  describe('Rendering', () => {
    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        version: '1.0.0',
        daemonStatus: 'up',
        daemonVersion: '2.0.0',
        networkEnv: 'mainnet',
        account: null,
      } as any);
    });

    it('renders info data container', () => {
      render(<InfoData />);

      expect(screen.getByTestId('info-data-container')).toBeInTheDocument();
    });

    it('renders client version', () => {
      render(<InfoData />);

      expect(
        screen.getByTestId('client-version-container'),
      ).toBeInTheDocument();
      expect(screen.getByTestId('client-version-label')).toHaveTextContent(
        'info.client version',
      );
      expect(screen.getByTestId('client-version-value')).toHaveTextContent(
        '1.0.0',
      );
    });

    it('renders daemon version when available', () => {
      render(<InfoData />);

      expect(
        screen.getByTestId('daemon-version-container'),
      ).toBeInTheDocument();
      expect(screen.getByTestId('daemon-version-label')).toHaveTextContent(
        'info.daemon version',
      );
      expect(screen.getByTestId('daemon-version-value')).toHaveTextContent(
        '2.0.0',
      );
    });

    it('renders network name when available', () => {
      render(<InfoData />);

      expect(screen.getByTestId('network-name-container')).toBeInTheDocument();
      expect(screen.getByTestId('network-name-label')).toHaveTextContent(
        'info.network name',
      );
      expect(screen.getByTestId('network-name-value')).toHaveTextContent(
        'mainnet',
      );
    });

    it('does not render daemon version when not available', () => {
      mockUseMainState.mockReturnValue({
        version: '1.0.0',
        daemonStatus: 'up',
        daemonVersion: null,
        networkEnv: 'mainnet',
        account: null,
      } as any);

      render(<InfoData />);

      expect(
        screen.queryByTestId('daemon-version-container'),
      ).not.toBeInTheDocument();
    });

    it('does not render network name when empty', () => {
      mockUseMainState.mockReturnValue({
        version: '1.0.0',
        daemonStatus: 'up',
        daemonVersion: '2.0.0',
        networkEnv: '',
        account: null,
      } as any);

      render(<InfoData />);

      expect(
        screen.queryByTestId('network-name-container'),
      ).not.toBeInTheDocument();
    });

    it('renders AccountData when account is available', () => {
      mockUseMainState.mockReturnValue({
        version: '1.0.0',
        daemonStatus: 'up',
        daemonVersion: '2.0.0',
        networkEnv: 'mainnet',
        account: { id: 'test-account' },
      } as any);

      render(<InfoData />);

      expect(screen.getByTestId('account-data-container')).toBeInTheDocument();
    });
  });

  describe('Copy Functionality', () => {
    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        version: '1.0.0',
        daemonStatus: 'up',
        daemonVersion: '2.0.0',
        networkEnv: 'mainnet',
        account: null,
      } as any);
    });

    it('copies client version when clicked', () => {
      render(<InfoData />);

      const clientVersionButton = screen.getByRole('button', { name: '1.0.0' });
      fireEvent.click(clientVersionButton);

      expect(mockCopy).toHaveBeenCalledWith('1.0.0', true);
    });

    it('copies daemon version when clicked', () => {
      render(<InfoData />);

      const daemonVersionButton = screen.getByRole('button', { name: '2.0.0' });
      fireEvent.click(daemonVersionButton);

      expect(mockCopy).toHaveBeenCalledWith('2.0.0');
    });

    it('copies network name when clicked', () => {
      render(<InfoData />);

      const networkNameButton = screen.getByRole('button', { name: 'mainnet' });
      fireEvent.click(networkNameButton);

      expect(mockCopy).toHaveBeenCalledWith('mainnet');
    });
  });

  describe('Dev Mode Functionality', () => {
    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        version: '1.0.0',
        daemonStatus: 'up',
        daemonVersion: '2.0.0',
        networkEnv: 'mainnet',
        account: null,
      } as any);
    });

    it('enables copy in dev mode', () => {
      Object.defineProperty(window, '_APP', {
        value: { devMode: true },
        writable: true,
      });

      render(<InfoData />);

      const clientVersionButton = screen.getByRole('button', { name: '1.0.0' });
      fireEvent.click(clientVersionButton);

      expect(mockCopy).toHaveBeenCalledWith('1.0.0', true);
    });
  });

  describe('Daemon Status Handling', () => {
    it('shows additional info when daemon is up', () => {
      mockUseMainState.mockReturnValue({
        version: '1.0.0',
        daemonStatus: 'up',
        daemonVersion: '2.0.0',
        networkEnv: 'mainnet',
        account: null,
      } as any);

      render(<InfoData />);

      expect(
        screen.getByTestId('daemon-version-container'),
      ).toBeInTheDocument();
      expect(screen.getByTestId('network-name-container')).toBeInTheDocument();
    });

    it('hides additional info when daemon is down', () => {
      mockUseMainState.mockReturnValue({
        version: '1.0.0',
        daemonStatus: 'down',
        daemonVersion: '2.0.0',
        networkEnv: 'mainnet',
        account: null,
      } as any);

      render(<InfoData />);

      expect(
        screen.queryByTestId('daemon-version-container'),
      ).not.toBeInTheDocument();
      expect(
        screen.queryByTestId('network-name-container'),
      ).not.toBeInTheDocument();
    });
  });

  describe('Version Display', () => {
    it('handles undefined version gracefully', () => {
      mockUseMainState.mockReturnValue({
        version: undefined,
        daemonStatus: 'up',
        daemonVersion: '2.0.0',
        networkEnv: 'mainnet',
        account: null,
      } as any);

      render(<InfoData />);

      const clientVersionButton = screen.getByTestId('client-version-value');
      expect(clientVersionButton).toHaveTextContent('');
    });

    it('displays version correctly', () => {
      mockUseMainState.mockReturnValue({
        version: '1.2.3-beta',
        daemonStatus: 'up',
        daemonVersion: '2.1.0',
        networkEnv: 'testnet',
        account: null,
      } as any);

      render(<InfoData />);

      expect(screen.getByTestId('client-version-value')).toHaveTextContent(
        '1.2.3-beta',
      );
      expect(screen.getByTestId('daemon-version-value')).toHaveTextContent(
        '2.1.0',
      );
      expect(screen.getByTestId('network-name-value')).toHaveTextContent(
        'testnet',
      );
    });
  });

  describe('Account Integration', () => {
    it('shows AccountData when account is present', () => {
      mockUseMainState.mockReturnValue({
        version: '1.0.0',
        daemonStatus: 'up',
        daemonVersion: '2.0.0',
        networkEnv: 'mainnet',
        account: { id: 'test-account-id' },
      } as any);

      render(<InfoData />);

      expect(screen.getByTestId('account-data-container')).toBeInTheDocument();
    });

    it('does not show AccountData when account is null', () => {
      mockUseMainState.mockReturnValue({
        version: '1.0.0',
        daemonStatus: 'up',
        daemonVersion: '2.0.0',
        networkEnv: 'mainnet',
        account: null,
      } as any);

      render(<InfoData />);

      expect(
        screen.queryByTestId('account-data-container'),
      ).not.toBeInTheDocument();
    });
  });
});
