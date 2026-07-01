import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import type { NetworkCompat } from '../../types';
import { initialState } from '../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../test/harness';
import Home from './Home';

// Home reads `window._APP.devMode` and the OS type at module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

// Home composes several heavy, independently-tested children. Stub them so this
// suite exercises Home's own composition/compat logic in isolation.
vi.mock('./TunnelState', () => ({
  TunnelState: () => <div data-testid="stub-tunnel-state" />,
}));
vi.mock('./NewBottomComponent', () => ({
  NewBottomComponent: () => <div data-testid="stub-bottom" />,
}));
vi.mock('./UpdateDialog', () => ({
  default: () => <div data-testid="stub-update-dialog" />,
}));
vi.mock('./GatewayIndependenceWarningDialog', () => ({
  default: () => <div data-testid="stub-gw-dialog" />,
}));
vi.mock('./NetworkUpdateDialog', () => ({
  default: ({ isOpen }: { isOpen: boolean }) => (
    <div data-testid="stub-network-dialog" data-open={String(isOpen)} />
  ),
}));

// The gateway-independence watcher (called by Home) reads a context hook.
vi.mock('../../hooks', () => ({
  useGatewayIndependenceWatcher: () => undefined,
}));

const compat = (core: boolean, tauri: boolean): NetworkCompat => ({
  core,
  tauri,
});

afterEach(() => {
  seedStore({ ...initialState });
});

describe('Home', () => {
  it('renders its core content', () => {
    renderWithProviders(<Home />);

    expect(screen.getByTestId('stub-tunnel-state')).toBeInTheDocument();
    expect(screen.getByTestId('stub-bottom')).toBeInTheDocument();
  });

  it('renders the network update dialog on non-windows platforms', () => {
    renderWithProviders(<Home />);

    expect(screen.getByTestId('stub-network-dialog')).toBeInTheDocument();
  });

  it('leaves the network update dialog closed when the versions are compatible', () => {
    seedStore({ networkCompat: compat(true, true) });

    renderWithProviders(<Home />);

    expect(screen.getByTestId('stub-network-dialog')).toHaveAttribute(
      'data-open',
      'false',
    );
  });

  it('opens the network update dialog when a component is incompatible', () => {
    seedStore({ networkCompat: compat(false, true) });

    renderWithProviders(<Home />);

    expect(screen.getByTestId('stub-network-dialog')).toHaveAttribute(
      'data-open',
      'true',
    );
  });
});
