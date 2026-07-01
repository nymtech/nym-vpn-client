import { describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { renderWithProviders } from '../../test/harness';
import WelcomeScreenContainer from './Container';

// `Container` pulls `ButtonIconNew` from the `../../ui` barrel, which loads
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

// The child views are exercised by their own tests; here we mock them to
// identifiable markers so we can assert the container's view-switching logic
// without pulling in their Tauri/deep-link dependencies.
vi.mock('./components/Welcome', () => ({
  Welcome: ({
    onSignup,
    onLogin,
  }: {
    onSignup: () => void;
    onLogin: () => void;
  }) => (
    <div>
      <span>welcome-view</span>
      <button onClick={onSignup}>go-signup</button>
      <button onClick={onLogin}>go-login</button>
    </div>
  ),
}));

vi.mock('./components/Signup', () => ({
  Signup: () => <span>signup-view</span>,
}));

vi.mock('./components/Login', () => ({
  Login: ({ onPassphrase }: { onPassphrase: () => void }) => (
    <div>
      <span>login-view</span>
      <button onClick={onPassphrase}>go-passphrase</button>
    </div>
  ),
}));

vi.mock('./components/PassphraseEnter', () => ({
  PassphraseEnter: () => <span>passphrase-view</span>,
}));

const navigate = vi.fn();
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useNavigate: () => navigate };
});

describe('WelcomeScreenContainer', () => {
  it('renders the welcome view by default without a back button', () => {
    renderWithProviders(<WelcomeScreenContainer />);

    expect(screen.getByText('welcome-view')).toBeInTheDocument();
    // `ButtonIconNew` renders the icon glyph as its inline text, so the back
    // button surfaces as an `arrow_back` button — absent on the welcome view.
    expect(
      screen.queryByRole('button', { name: 'arrow_back' }),
    ).not.toBeInTheDocument();
  });

  it('navigates from welcome to the signup view', async () => {
    renderWithProviders(<WelcomeScreenContainer />);

    await userEvent.click(screen.getByRole('button', { name: 'go-signup' }));

    // `AnimatePresence mode="wait"` swaps views after the exit transition, so
    // wait for the new view to settle in.
    expect(await screen.findByText('signup-view')).toBeInTheDocument();
  });

  it('navigates welcome → login → passphrase and back to login', async () => {
    renderWithProviders(<WelcomeScreenContainer />);

    await userEvent.click(screen.getByRole('button', { name: 'go-login' }));
    expect(await screen.findByText('login-view')).toBeInTheDocument();

    await userEvent.click(
      screen.getByRole('button', { name: 'go-passphrase' }),
    );
    expect(await screen.findByText('passphrase-view')).toBeInTheDocument();

    // The back button (rendered as an `arrow_back` glyph) returns to login.
    await userEvent.click(screen.getByRole('button', { name: 'arrow_back' }));
    await waitFor(() =>
      expect(screen.getByText('login-view')).toBeInTheDocument(),
    );
  });

  it('routes to root when the close button is clicked', async () => {
    renderWithProviders(<WelcomeScreenContainer />);

    await userEvent.click(screen.getByRole('button', { name: 'close' }));

    expect(navigate).toHaveBeenCalledWith('/home');
  });
});
