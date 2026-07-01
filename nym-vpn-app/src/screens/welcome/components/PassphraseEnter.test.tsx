import type { ReactElement } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Toast } from '@base-ui/react';

import {
  mockTauriCommands,
  renderWithProviders,
  seedStore,
} from '../../../test/harness';
import { PassphraseEnter } from './PassphraseEnter';

// `PassphraseEnter` pulls `Button`/`TextArea` from the `../../../ui` barrel,
// which loads `DaemonDot` reading `window._APP.devMode` at module-load time;
// `vi.hoisted` runs before the static imports below so the global exists.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const navigate = vi.fn();
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useNavigate: () => navigate };
});

// `useToast` reads from a base-ui toast manager, which needs a `Toast.Provider`.
function renderPassphrase(ui: ReactElement) {
  return renderWithProviders(<Toast.Provider>{ui}</Toast.Provider>);
}

describe('PassphraseEnter', () => {
  beforeEach(() => {
    navigate.mockReset();
  });

  it('renders the title and login button', () => {
    seedStore({ daemonStatus: 'ok', state: 'disconnected' });
    renderPassphrase(<PassphraseEnter />);

    expect(screen.getByRole('heading', { name: 'Log in' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Login' })).toBeInTheDocument();
  });

  it('disables the login button while the daemon is down', () => {
    seedStore({ daemonStatus: 'down', state: 'disconnected' });
    renderPassphrase(<PassphraseEnter />);

    expect(screen.getByRole('button', { name: 'Login' })).toBeDisabled();
  });

  it('logs in and navigates to the technical opt-in when unseen', async () => {
    seedStore({
      daemonStatus: 'ok',
      state: 'disconnected',
      technicalOptinSeen: false,
    });
    mockTauriCommands((cmd) => {
      if (cmd === 'get_account_mode') return 'api';
      return null;
    });

    renderPassphrase(<PassphraseEnter />);

    const textarea = screen.getByRole('textbox');
    await userEvent.type(textarea, 'twenty four words go here');
    await userEvent.click(screen.getByRole('button', { name: 'Login' }));

    await waitFor(() =>
      expect(navigate).toHaveBeenCalledWith('/technical-optin'),
    );
  });

  it('navigates to root after login when the opt-in was already seen', async () => {
    seedStore({
      daemonStatus: 'ok',
      state: 'disconnected',
      technicalOptinSeen: true,
    });
    mockTauriCommands((cmd) => {
      if (cmd === 'get_account_mode') return 'api';
      return null;
    });

    renderPassphrase(<PassphraseEnter />);

    await userEvent.type(screen.getByRole('textbox'), 'my passphrase');
    await userEvent.click(screen.getByRole('button', { name: 'Login' }));

    await waitFor(() => expect(navigate).toHaveBeenCalledWith('/home'));
  });
});
