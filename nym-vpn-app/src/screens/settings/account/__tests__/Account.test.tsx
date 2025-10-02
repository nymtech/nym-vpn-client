import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { render, mockLoginContexts } from '../../../../test/test-utils';
import Account from '../Account';
import { useMainState, useMainDispatch } from '../../../../contexts';

const mockInvoke = invoke as jest.MockedFunction<typeof invoke>;
const mockOpenUrl = openUrl as jest.MockedFunction<typeof openUrl>;
const mockUseMainState = useMainState as jest.MockedFunction<
  typeof useMainState
>;
const mockUseMainDispatch = useMainDispatch as jest.MockedFunction<
  typeof useMainDispatch
>;

const mockDispatch = jest.fn();

jest.mock('@tauri-apps/plugin-opener');

describe('Account Component', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockUseMainDispatch.mockReturnValue(mockDispatch);
  });

  describe('When user is not logged in', () => {
    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: null,
        accountState: null,
        accountSyncing: false,
        accountLinks: null,
      } as any);
    });

    it('renders login button when no account', () => {
      render(<Account />);

      const loginButton = screen.getByRole('button', { name: 'login button' });
      expect(loginButton).toBeInTheDocument();
    });

    it('disables login button when daemon is down', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'down',
        account: null,
        accountState: null,
        accountSyncing: false,
        accountLinks: null,
      } as any);

      render(<Account />);

      const loginButton = screen.getByRole('button', { name: 'login button' });
      expect(loginButton).toBeDisabled();
    });
  });

  describe('When user is logged in', () => {
    const mockAccount = {
      id: 'test-account-id',
      email: 'test@example.com',
    };

    const mockAccountLinks = {
      account: 'https://account.nym.com',
      signIn: 'https://signin.nym.com',
    };

    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountState: 'active',
        accountSyncing: false,
        accountLinks: mockAccountLinks,
      } as any);
    });

    it('renders account menu card when logged in', () => {
      render(<Account />);

      expect(screen.getByText('Account')).toBeInTheDocument();
      expect(screen.getByTestId('settings-card-account')).toBeInTheDocument();
    });

    it('shows syncing state when account is syncing', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountState: 'active',
        accountSyncing: true,
        accountLinks: mockAccountLinks,
      } as any);

      render(<Account />);

      expect(screen.getByText('account.syncing')).toBeInTheDocument();
    });

    it('shows get started button when no subscription', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountState: 'no-subscription',
        accountSyncing: false,
        accountLinks: mockAccountLinks,
      } as any);

      render(<Account />);

      expect(
        screen.getByRole('button', { name: 'account.get started' }),
      ).toBeInTheDocument();
    });

    it('shows get started button when bandwidth exceeded', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountState: 'bandwidth-exceeded',
        accountSyncing: false,
        accountLinks: mockAccountLinks,
      } as any);

      render(<Account />);

      expect(
        screen.getByRole('button', { name: 'account.get started' }),
      ).toBeInTheDocument();
    });

    it('disables get started button when daemon is down', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'down',
        account: mockAccount,
        accountState: 'no-subscription',
        accountSyncing: false,
        accountLinks: mockAccountLinks,
      } as any);

      render(<Account />);

      const getStartedButton = screen.getByRole('button', {
        name: 'account.get started',
      });
      expect(getStartedButton).toBeDisabled();
    });

    it('disables get started button when syncing', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountState: 'no-subscription',
        accountSyncing: true,
        accountLinks: mockAccountLinks,
      } as any);

      render(<Account />);

      const getStartedButton = screen.getByRole('button', {
        name: 'account.get started',
      });
      expect(getStartedButton).toBeDisabled();
    });

    it('opens account URL when account menu is clicked', async () => {
      render(<Account />);

      const accountCard = screen.getByTestId('settings-card-account');
      fireEvent.click(accountCard);

      await waitFor(() => {
        expect(mockOpenUrl).toHaveBeenCalledWith('https://account.nym.com');
      });
    });

    it('opens sign in URL when account URL is not available', async () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountState: 'active',
        accountSyncing: false,
        accountLinks: { account: null, signIn: 'https://signin.nym.com' },
      } as any);

      render(<Account />);

      const accountCard = screen.getByTestId('settings-card-account');
      fireEvent.click(accountCard);

      await waitFor(() => {
        expect(mockOpenUrl).toHaveBeenCalledWith('https://signin.nym.com');
      });
    });

    it('disables account menu when no URLs are available', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountState: 'active',
        accountSyncing: false,
        accountLinks: { account: null, signIn: null },
      } as any);

      render(<Account />);

      const accountCard = screen.getByTestId('settings-card-account');
      expect(accountCard).toHaveAttribute('data-test-disabled', 'true');
    });
  });

  describe('Account state descriptions', () => {
    const mockAccount = { id: 'test-account-id' };
    const mockAccountLinks = { account: 'https://account.nym.com' };

    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountSyncing: false,
        accountLinks: mockAccountLinks,
      } as any);
    });

    it('shows correct description for no-subscription state', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountState: 'no-subscription',
        accountSyncing: false,
        accountLinks: mockAccountLinks,
      } as any);

      render(<Account />);

      expect(screen.getByText('account.no plan')).toBeInTheDocument();
    });

    it('shows correct description for max-device-reached state', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountState: 'max-device-reached',
        accountSyncing: false,
        accountLinks: mockAccountLinks,
      } as any);

      render(<Account />);

      expect(
        screen.getByText('account.max device reached'),
      ).toBeInTheDocument();
    });

    it('shows correct description for status-not-active state', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountState: 'status-not-active',
        accountSyncing: false,
        accountLinks: mockAccountLinks,
      } as any);

      render(<Account />);

      expect(screen.getByText('account.status inactive')).toBeInTheDocument();
    });

    it('shows correct description for bandwidth-exceeded state', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountState: 'bandwidth-exceeded',
        accountSyncing: false,
        accountLinks: mockAccountLinks,
      } as any);

      render(<Account />);

      expect(
        screen.getByText('account.bandwidth exceeded'),
      ).toBeInTheDocument();
    });

    it('shows correct description for error state', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: mockAccount,
        accountState: 'error',
        accountSyncing: false,
        accountLinks: mockAccountLinks,
      } as any);

      render(<Account />);

      expect(screen.getByText('account.error')).toBeInTheDocument();
    });
  });

  describe('Account check on mount', () => {
    it('checks account status when daemon is up', async () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        account: null,
        accountState: null,
        accountSyncing: false,
        accountLinks: null,
      } as any);

      mockInvoke.mockResolvedValue(true);

      render(<Account />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('is_account_stored');
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'set-account',
          stored: true,
        });
      });
    });

    it('does not check account status when daemon is down', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'down',
        account: null,
        accountState: null,
        accountSyncing: false,
        accountLinks: null,
      } as any);

      render(<Account />);

      expect(mockInvoke).not.toHaveBeenCalled();
    });
  });
});
