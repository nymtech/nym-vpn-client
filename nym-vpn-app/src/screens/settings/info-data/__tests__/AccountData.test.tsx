import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import AccountData from '../AccountData';
import { useMainState } from '../../../../contexts';
import { useClipboard } from '../../../../hooks';

const mockInvoke = invoke as jest.MockedFunction<typeof invoke>;
const mockUseMainState = useMainState as jest.MockedFunction<
  typeof useMainState
>;
const mockUseClipboard = useClipboard as jest.MockedFunction<
  typeof useClipboard
>;

const mockCopy = jest.fn() as jest.MockedFunction<
  (text: string, notify?: boolean) => Promise<void>
>;

jest.mock('../../../../cache', () => ({
  CCache: {
    get: jest.fn(),
    set: jest.fn(),
  },
}));

describe('AccountData Component', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockUseClipboard.mockReturnValue({ copy: mockCopy });
  });

  describe('When user is not logged in', () => {
    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        account: null,
      } as any);
    });

    it('returns null when no account', () => {
      const { container } = render(<AccountData />);
      expect(container.firstChild).toBeNull();
    });
  });

  describe('When user is logged in', () => {
    const mockAccount = { id: 'test-account-id' };

    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        account: mockAccount,
      } as any);
    });

    it('renders account data container', () => {
      render(<AccountData />);

      expect(screen.getByTestId('account-data-container')).toBeInTheDocument();
    });

    it('loads account ID from cache first', async () => {
      const { CCache } = require('../../../../cache');
      CCache.get.mockResolvedValue('cached-account-id');

      render(<AccountData />);

      await waitFor(() => {
        expect(CCache.get).toHaveBeenCalledWith('cache-account-id');
        expect(
          screen.getByTestId('account-id-value-content'),
        ).toHaveTextContent('cached-a…count-id');
      });
    });

    it('fetches account ID from API when not in cache', async () => {
      const { CCache } = require('../../../../cache');
      CCache.get.mockResolvedValue(null);
      mockInvoke.mockResolvedValue('api-account-id');

      render(<AccountData />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_account_id');
        expect(
          screen.getByTestId('account-id-value-content'),
        ).toHaveTextContent('api-account-id');
      });
    });

    it('caches account ID after fetching from API', async () => {
      const { CCache } = require('../../../../cache');
      CCache.get.mockResolvedValue(null);
      mockInvoke.mockResolvedValue('api-account-id');

      render(<AccountData />);

      await waitFor(() => {
        expect(CCache.set).toHaveBeenCalledWith(
          'cache-account-id',
          'api-account-id',
          120,
        );
      });
    });

    it('loads device ID from cache first', async () => {
      const { CCache } = require('../../../../cache');
      CCache.get.mockResolvedValue('cached-device-id');

      render(<AccountData />);

      await waitFor(() => {
        expect(CCache.get).toHaveBeenCalledWith('cache-device-id');
        expect(screen.getByTestId('device-id-value-content')).toHaveTextContent(
          'cached-d…evice-id',
        );
      });
    });

    it('fetches device ID from API when not in cache', async () => {
      const { CCache } = require('../../../../cache');
      CCache.get.mockResolvedValue(null);
      mockInvoke.mockResolvedValue('api-device-id');

      render(<AccountData />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_device_id');
        expect(screen.getByTestId('device-id-value-content')).toHaveTextContent(
          'api-device-id',
        );
      });
    });

    it('caches device ID after fetching from API', async () => {
      const { CCache } = require('../../../../cache');
      CCache.get.mockResolvedValue(null);
      mockInvoke.mockResolvedValue('api-device-id');

      render(<AccountData />);

      await waitFor(() => {
        expect(CCache.set).toHaveBeenCalledWith(
          'cache-device-id',
          'api-device-id',
          120,
        );
      });
    });

    it('handles API errors gracefully', async () => {
      const { CCache } = require('../../../../cache');
      CCache.get.mockResolvedValue(null);
      mockInvoke.mockRejectedValue(new Error('API error'));

      render(<AccountData />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_account_id');
        expect(
          screen.queryByTestId('account-id-container'),
        ).not.toBeInTheDocument();
      });
    });
  });

  describe('ID Display and Copy Functionality', () => {
    const mockAccount = { id: 'test-account-id' };

    beforeEach(() => {
      mockUseMainState.mockReturnValue({
        account: mockAccount,
      } as any);
    });

    it('displays account ID when available', async () => {
      const { CCache } = require('../../../../cache');
      CCache.get.mockResolvedValue('test-account-id-123');

      render(<AccountData />);

      await waitFor(() => {
        expect(screen.getByTestId('account-id-container')).toBeInTheDocument();
        expect(screen.getByTestId('account-id-label')).toHaveTextContent(
          'info.account id',
        );
        expect(
          screen.getByTestId('account-id-value-content'),
        ).toHaveTextContent('test-acc…t-id-123');
      });
    });

    it('displays device ID when available', async () => {
      const { CCache } = require('../../../../cache');
      CCache.get
        .mockResolvedValueOnce(null) // account ID not in cache
        .mockResolvedValueOnce('test-device-id-456'); // device ID in cache
      mockInvoke.mockResolvedValue(null); // account ID API returns null

      render(<AccountData />);

      await waitFor(() => {
        expect(screen.getByTestId('device-id-container')).toBeInTheDocument();
        expect(screen.getByTestId('device-id-label')).toHaveTextContent(
          'info.device id',
        );
        expect(screen.getByTestId('device-id-value-content')).toHaveTextContent(
          'test-dev…e-id-456',
        );
      });
    });

    it('copies account ID when clicked', async () => {
      const { CCache } = require('../../../../cache');
      CCache.get.mockResolvedValue('test-account-id-123');

      render(<AccountData />);

      await waitFor(() => {
        const accountIdButton = screen.getByTestId('account-id-value');
        fireEvent.click(accountIdButton);
        expect(mockCopy).toHaveBeenCalledWith('test-account-id-123');
      });
    });

    it('does not display account ID when not available', async () => {
      const { CCache } = require('../../../../cache');
      CCache.get.mockResolvedValue(null);
      mockInvoke.mockResolvedValue(null);

      render(<AccountData />);

      await waitFor(() => {
        expect(
          screen.queryByTestId('account-id-container'),
        ).not.toBeInTheDocument();
      });
    });
  });

  describe('Effect Dependencies', () => {
    it('refetches data when account changes', async () => {
      const { CCache } = require('../../../../cache');
      CCache.get.mockResolvedValue(null);
      mockInvoke.mockResolvedValue('initial-account-id');

      mockUseMainState.mockReturnValue({
        account: { id: 'initial-account-id' },
      } as any);

      const { rerender } = render(<AccountData />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_account_id');
      });

      mockUseMainState.mockReturnValue({
        account: { id: 'new-account-id' },
      } as any);

      rerender(<AccountData />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledTimes(2);
      });
    });
  });
});
