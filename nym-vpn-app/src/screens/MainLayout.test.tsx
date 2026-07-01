import type { ReactNode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { Route, Routes } from 'react-router';
import { initialState } from '../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../test/harness';
import MainLayout from './MainLayout';

// The `../ui` barrel loads `DaemonDot` (reads `window._APP.devMode`) and the
// Tauri OS plugin at module-load time; the hoisted global + OS mock satisfy
// both before the static imports run.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

// `MainLayout` only composes chrome around an `<Outlet/>`; each child is
// independently tested, so stub them to identifiable markers to isolate the
// layout's own composition/toggle logic from their Tauri/context dependencies.
vi.mock('../ui', () => ({
  TopBar: () => <div data-testid="stub-topbar" />,
  DaemonDot: () => <div data-testid="stub-daemon-dot" />,
}));

vi.mock('../layers', () => ({
  EventNotification: ({ children }: { children: ReactNode }) => (
    <div data-testid="stub-event-notification">{children}</div>
  ),
}));

vi.mock('../components/toast', () => ({
  ToastList: () => <div data-testid="stub-toast-list" />,
}));

vi.mock('../contexts/CardAnimationContext', () => ({
  CardAnimationProvider: ({ children }: { children: ReactNode }) => (
    <div data-testid="stub-card-animation">{children}</div>
  ),
}));

vi.mock('./SystemAuthentication', () => ({
  SystemAuthentication: () => <div data-testid="stub-system-auth" />,
}));

function renderLayout(props: Parameters<typeof MainLayout>[0] = {}) {
  return renderWithProviders(
    <Routes>
      <Route element={<MainLayout {...props} />}>
        <Route index element={<div data-testid="outlet-child" />} />
      </Route>
    </Routes>,
  );
}

afterEach(() => {
  seedStore({ ...initialState });
});

describe('MainLayout', () => {
  it('mounts the routed child through the outlet', () => {
    renderLayout();

    expect(screen.getByTestId('outlet-child')).toBeInTheDocument();
    expect(screen.getByTestId('stub-event-notification')).toBeInTheDocument();
  });

  it('renders the top bar, daemon dot, toasts and system auth by default', () => {
    renderLayout();

    expect(screen.getByTestId('stub-topbar')).toBeInTheDocument();
    expect(screen.getByTestId('stub-daemon-dot')).toBeInTheDocument();
    expect(screen.getByTestId('stub-toast-list')).toBeInTheDocument();
    expect(screen.getByTestId('stub-system-auth')).toBeInTheDocument();
  });

  it('hides the top bar when noTopBar is set', () => {
    renderLayout({ noTopBar: true });

    expect(screen.queryByTestId('stub-topbar')).not.toBeInTheDocument();
    expect(screen.getByTestId('outlet-child')).toBeInTheDocument();
  });

  it('hides the daemon dot when noDaemonDot is set', () => {
    renderLayout({ noDaemonDot: true });

    expect(screen.queryByTestId('stub-daemon-dot')).not.toBeInTheDocument();
  });

  it('hides notifications when noNotifications is set', () => {
    renderLayout({ noNotifications: true });

    expect(screen.queryByTestId('stub-toast-list')).not.toBeInTheDocument();
  });
});
