import React from 'react';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { invoke } from '@tauri-apps/api/core';
import { render } from '../../../test/test-utils';
import Login from '../Login';
import {
  useMainState,
  useMainDispatch,
  useInAppNotify,
} from '../../../contexts';
import { useI18nError } from '../../../hooks';

const mockNavigate = jest.fn();
const mockDispatch = jest.fn();
const mockPush = jest.fn();
const mockTe = jest.fn((error: any) =>
  typeof error === 'string' ? error : error.key || 'error',
);

jest.mock('react-router', () => ({
  ...(jest.requireActual('react-router') as any),
  useNavigate: () => mockNavigate,
}));

jest.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        welcome: 'Welcome to Nym VPN',
        description: 'Enter your account credentials to continue',
        'input-label': 'Mnemonic phrase',
        'input-placeholder': 'Enter your mnemonic phrase here...',
        'login-button': 'Login',
        'added-notification': 'Account added successfully',
        'create-account.text': "Don't have an account?",
        'create-account.link': 'Create one',
      };
      return translations[key] || key;
    },
  }),
}));

const mockUseMainState = useMainState as jest.MockedFunction<
  typeof useMainState
>;
const mockUseMainDispatch = useMainDispatch as jest.MockedFunction<
  typeof useMainDispatch
>;
const mockUseInAppNotify = useInAppNotify as jest.MockedFunction<
  typeof useInAppNotify
>;
const mockUseI18nError = useI18nError as jest.MockedFunction<
  typeof useI18nError
>;
const mockInvoke = invoke as jest.MockedFunction<typeof invoke>;

describe('Login Component', () => {
  beforeEach(() => {
    jest.clearAllMocks();

    mockUseMainState.mockReturnValue({
      daemonStatus: 'up',
      state: 'disconnected',
    } as any);

    mockUseMainDispatch.mockReturnValue(mockDispatch);

    mockUseInAppNotify.mockReturnValue({
      push: mockPush,
    } as any);

    mockUseI18nError.mockReturnValue({
      tE: mockTe,
    });
  });

  describe('Rendering', () => {
    it('renders all main elements correctly', () => {
      render(<Login />);

      expect(screen.getByTestId('login-page')).toBeInTheDocument();
      expect(screen.getByTestId('login-welcome-text')).toHaveTextContent(
        'Welcome to Nym VPN',
      );
      expect(screen.getByTestId('login-description')).toHaveTextContent(
        'Enter your account credentials to continue',
      );
      expect(screen.getByTestId('login-mnemonic-input')).toBeInTheDocument();
      expect(screen.getByTestId('login-submit-button')).toBeInTheDocument();
      expect(
        screen.getByTestId('login-create-account-section'),
      ).toBeInTheDocument();
    });

    it('shows correct placeholder and label for input', () => {
      render(<Login />);

      const input = screen.getByTestId('login-mnemonic-input');
      expect(input).toHaveAttribute(
        'placeholder',
        'Enter your mnemonic phrase here...',
      );
    });

    it('shows create account link', () => {
      render(<Login />);

      expect(screen.getByTestId('login-create-account-text')).toHaveTextContent(
        "Don't have an account?",
      );
      expect(
        screen.getByTestId('login-create-account-link'),
      ).toBeInTheDocument();
    });
  });

  describe('Form Input', () => {
    it('updates phrase value when typing', async () => {
      const user = userEvent.setup();
      render(<Login />);

      const input = screen.getByTestId('login-mnemonic-input');
      await user.type(input, 'test mnemonic phrase');

      expect(input).toHaveValue('test mnemonic phrase');
    });

    it('clears error when input becomes empty', async () => {
      const user = userEvent.setup();
      render(<Login />);

      const input = screen.getByTestId('login-mnemonic-input');

      await user.type(input, 'test');

      await user.clear(input);

      expect(
        screen.queryByTestId('login-error-message'),
      ).not.toBeInTheDocument();
    });
  });

  describe('Button States', () => {
    it('enables submit button when daemon is up and state is disconnected', () => {
      render(<Login />);

      const button = screen.getByTestId('login-submit-button');
      expect(button).not.toBeDisabled();
    });

    it('disables submit button when daemon is down', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'down',
        state: 'disconnected',
      } as any);

      render(<Login />);

      const button = screen.getByTestId('login-submit-button');
      expect(button).toBeDisabled();
    });

    it('disables submit button when state is not disconnected', () => {
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        state: 'connected',
      } as any);

      render(<Login />);

      const button = screen.getByTestId('login-submit-button');
      expect(button).toBeDisabled();
    });

    it('shows loading spinner when submitting', async () => {
      const user = userEvent.setup();
      mockInvoke.mockImplementation(() => new Promise(() => {}));

      render(<Login />);

      const input = screen.getByTestId('login-mnemonic-input');
      const button = screen.getByTestId('login-submit-button');

      await user.type(input, 'test mnemonic phrase');
      await user.click(button);

      expect(screen.getByTestId('button-spinner')).toBeInTheDocument();
    });
  });

  describe('Form Submission', () => {
    it('does not submit when phrase is empty', async () => {
      const user = userEvent.setup();
      render(<Login />);

      const button = screen.getByTestId('login-submit-button');
      await user.click(button);

      expect(mockInvoke).not.toHaveBeenCalled();
    });

    it('does not submit when already loading', async () => {
      const user = userEvent.setup();
      mockInvoke.mockImplementation(() => new Promise(() => {}));

      render(<Login />);

      const input = screen.getByTestId('login-mnemonic-input');
      const button = screen.getByTestId('login-submit-button');

      await user.type(input, 'test mnemonic phrase');

      await user.click(button);
      await user.click(button);

      expect(mockInvoke).toHaveBeenCalledTimes(1);
    });

    it('does not submit when state is not disconnected', async () => {
      const user = userEvent.setup();
      mockUseMainState.mockReturnValue({
        daemonStatus: 'up',
        state: 'connected',
      } as any);

      render(<Login />);

      const input = screen.getByTestId('login-mnemonic-input');
      const button = screen.getByTestId('login-submit-button');

      await user.type(input, 'test mnemonic phrase');
      await user.click(button);

      expect(mockInvoke).not.toHaveBeenCalled();
    });

    it('submits with correct parameters when form is valid', async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue(12345);

      render(<Login />);

      const input = screen.getByTestId('login-mnemonic-input');
      const button = screen.getByTestId('login-submit-button');

      await user.type(input, '  test mnemonic phrase  ');
      await user.click(button);

      expect(mockInvoke).toHaveBeenCalledWith('add_account', {
        mnemonic: 'test mnemonic phrase',
      });
    });
  });

  describe('Success Flow', () => {
    it('handles successful login correctly', async () => {
      const user = userEvent.setup();
      mockInvoke.mockResolvedValue(12345);

      render(<Login />);

      const input = screen.getByTestId('login-mnemonic-input');
      const button = screen.getByTestId('login-submit-button');

      await user.type(input, 'test mnemonic phrase');
      await user.click(button);

      await waitFor(() => {
        expect(mockNavigate).toHaveBeenCalledWith('/');
        expect(mockDispatch).toHaveBeenCalledWith({
          type: 'set-account',
          stored: true,
        });
        expect(mockPush).toHaveBeenCalledWith({
          message: 'Account added successfully',
          close: true,
        });
        expect(mockDispatch).toHaveBeenCalledWith({ type: 'reset-error' });
      });
    });
  });

  describe('Error Handling', () => {
    it('displays error message when login fails', async () => {
      const user = userEvent.setup();
      const backendError = {
        key: 'INVALID_MNEMONIC',
        data: { reason: 'Invalid format' },
      };
      mockInvoke.mockRejectedValue(backendError);

      render(<Login />);

      const input = screen.getByTestId('login-mnemonic-input');
      const button = screen.getByTestId('login-submit-button');

      await user.type(input, 'invalid mnemonic');
      await user.click(button);

      await waitFor(() => {
        const errorMessage = screen.getByTestId('login-error-message');
        expect(errorMessage).toBeInTheDocument();
        expect(errorMessage).toHaveTextContent(': Invalid format');
      });

      expect(mockTe).toHaveBeenCalledWith('INVALID_MNEMONIC');
    });

    it('stops loading state after error', async () => {
      const user = userEvent.setup();
      mockInvoke.mockRejectedValue({ key: 'ERROR' });

      render(<Login />);

      const input = screen.getByTestId('login-mnemonic-input');
      const button = screen.getByTestId('login-submit-button');

      await user.type(input, 'test mnemonic');
      await user.click(button);

      await waitFor(() => {
        expect(screen.getByTestId('login-error-message')).toBeInTheDocument();
      });

      expect(button).not.toHaveAttribute('data-spinner', 'true');
    });
  });
});
