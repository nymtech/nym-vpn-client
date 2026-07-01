import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { renderWithProviders } from '../../../test/harness';
import { Welcome } from './Welcome';

// `Welcome` pulls `Button` from the `../../../ui` barrel, which loads
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

describe('Welcome', () => {
  it('renders the title, description and both action buttons', () => {
    renderWithProviders(<Welcome onSignup={vi.fn()} onLogin={vi.fn()} />);

    expect(
      screen.getByRole('heading', { name: 'Welcome!' }),
    ).toBeInTheDocument();
    expect(
      screen.getByText('To the world’s most private VPN.'),
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Sign up' })).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Login to my account' }),
    ).toBeInTheDocument();
  });

  it('fires onSignup when the sign-up button is clicked', async () => {
    const onSignup = vi.fn();
    renderWithProviders(<Welcome onSignup={onSignup} onLogin={vi.fn()} />);

    await userEvent.click(screen.getByRole('button', { name: 'Sign up' }));

    expect(onSignup).toHaveBeenCalledOnce();
  });

  it('fires onLogin when the login button is clicked', async () => {
    const onLogin = vi.fn();
    renderWithProviders(<Welcome onSignup={vi.fn()} onLogin={onLogin} />);

    await userEvent.click(
      screen.getByRole('button', { name: 'Login to my account' }),
    );

    expect(onLogin).toHaveBeenCalledOnce();
  });
});
