import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { initialState } from '../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../test/harness';
import StartupGate from './StartupGate';

// `StartupGate` imports `../router`, which transitively loads the `../ui`
// barrel (`DaemonDot` reads `window._APP.devMode`) and the Tauri OS plugin at
// module-load time; the hoisted global + OS mock satisfy both before import.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

// `StartupGate` renders a `<Navigate>` for every branch; stub it to a marker
// that surfaces the resolved destination so we can assert the redirect target.
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return {
    ...actual,
    Navigate: ({ to }: { to: string }) => (
      <div data-testid="navigate" data-to={to} />
    ),
  };
});

afterEach(() => {
  seedStore({ ...initialState });
});

function destination() {
  return screen.getByTestId('navigate').getAttribute('data-to');
}

describe('StartupGate', () => {
  it('redirects to root when the daemon is down', () => {
    seedStore({ daemonStatus: 'down', account: true });

    renderWithProviders(<StartupGate />);

    expect(destination()).toBe('/home');
  });

  it('redirects to root when authentication was denied', () => {
    seedStore({ daemonStatus: 'auth-denied', account: false });

    renderWithProviders(<StartupGate />);

    expect(destination()).toBe('/home');
  });

  it('redirects to onboarding when there is no account', () => {
    seedStore({ daemonStatus: 'ok', account: false });

    renderWithProviders(<StartupGate />);

    expect(destination()).toBe('/hideout/onboarding');
  });

  it('redirects to root when an account exists and the daemon is ok', () => {
    seedStore({ daemonStatus: 'ok', account: true });

    renderWithProviders(<StartupGate />);

    expect(destination()).toBe('/home');
  });
});
