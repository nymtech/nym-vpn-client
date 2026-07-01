import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { renderWithProviders } from '../../../test/harness';
import { Login } from './Login';

// `Login` pulls `Button` from the `../../../ui` barrel, which loads `DaemonDot`
// reading `window._APP.devMode` at module-load time; `vi.hoisted` runs before
// the static imports below so the global exists in time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

// `PrivyButton` drives its own Tauri deep-link flow (covered by its own test);
// stub it to a labelled marker so this screen stays isolated.
vi.mock('../../../components', () => ({
  PrivyButton: ({ label }: { label: string }) => <button>{label}</button>,
}));

describe('Login', () => {
  it('renders the title and the 24-word login button', () => {
    renderWithProviders(<Login onPassphrase={vi.fn()} />);

    expect(screen.getByRole('heading', { name: 'Log in' })).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Login with 24 words' }),
    ).toBeInTheDocument();
  });

  it('renders the social login button via PrivyButton', () => {
    renderWithProviders(<Login onPassphrase={vi.fn()} />);

    expect(
      screen.getByRole('button', { name: 'Login using linked socials*' }),
    ).toBeInTheDocument();
  });

  it('fires onPassphrase when the 24-word button is clicked', async () => {
    const onPassphrase = vi.fn();
    renderWithProviders(<Login onPassphrase={onPassphrase} />);

    await userEvent.click(
      screen.getByRole('button', { name: 'Login with 24 words' }),
    );

    expect(onPassphrase).toHaveBeenCalledOnce();
  });
});
