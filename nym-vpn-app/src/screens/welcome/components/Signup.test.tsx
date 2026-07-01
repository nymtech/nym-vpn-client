import { beforeEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import {
  mockTauriCommands,
  renderWithProviders,
  seedStore,
} from '../../../test/harness';
import { Signup } from './Signup';

// `Signup` pulls `Button` from the `../../../ui` barrel, which loads
// `DaemonDot` reading `window._APP.devMode` at module-load time; `vi.hoisted`
// runs before the static imports below so the global exists in time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const openUrl = vi.fn();
vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: (url: string): void => {
    openUrl(url);
  },
}));

const startListening = vi.fn<(timeoutMs?: number) => Promise<string>>();
vi.mock('../../../hooks', () => ({
  useDeepLink: (): { startListening: typeof startListening } => ({
    startListening,
  }),
}));

// `PrivyButton` drives its own Tauri deep-link flow (covered by its own test);
// stub it to a labelled marker so this screen stays isolated.
vi.mock('../../../components', () => ({
  PrivyButton: ({ label }: { label: string }) => <button>{label}</button>,
}));

const navigate = vi.fn();
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useNavigate: () => navigate };
});

describe('Signup', () => {
  beforeEach(() => {
    openUrl.mockReset();
    startListening.mockReset();
    navigate.mockReset();
    mockTauriCommands((cmd) =>
      cmd === 'get_deep_link' ? 'https://create.example' : null,
    );
  });

  it('renders the title and both signup buttons', () => {
    renderWithProviders(<Signup />);

    expect(
      screen.getByRole('heading', { name: 'Sign Up' }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Anonymous account' }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Sign up with socials*' }),
    ).toBeInTheDocument();
  });

  it('opens the create-account deep link on click', async () => {
    // Never resolves within the test so we stay in the pending branch.
    startListening.mockReturnValue(new Promise<string>(() => undefined));

    renderWithProviders(<Signup />);

    await userEvent.click(
      screen.getByRole('button', { name: 'Anonymous account' }),
    );

    await waitFor(() =>
      expect(openUrl).toHaveBeenCalledWith('https://create.example'),
    );
    expect(startListening).toHaveBeenCalledOnce();
  });

  it('navigates to the technical opt-in when it has not yet been seen', async () => {
    seedStore({ technicalOptinSeen: false });
    startListening.mockResolvedValue('nym://callback');

    renderWithProviders(<Signup />);

    await userEvent.click(
      screen.getByRole('button', { name: 'Anonymous account' }),
    );

    await waitFor(() =>
      expect(navigate).toHaveBeenCalledWith('/technical-optin'),
    );
  });

  it('navigates to root when technical opt-in has already been seen', async () => {
    seedStore({ technicalOptinSeen: true });
    startListening.mockResolvedValue('nym://callback');

    renderWithProviders(<Signup />);

    await userEvent.click(
      screen.getByRole('button', { name: 'Anonymous account' }),
    );

    await waitFor(() => expect(navigate).toHaveBeenCalledWith('/home'));
  });
});
