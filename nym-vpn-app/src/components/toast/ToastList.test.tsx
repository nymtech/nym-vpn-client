import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { act, useEffect } from 'react';
import { screen, waitFor } from '@testing-library/react';
import { Toast } from '@base-ui/react';

import { renderWithProviders } from '../../test/harness';
import ToastList from './ToastList';

// `ToastList` pulls `MsIcon` from the `../../ui` barrel, which loads `DaemonDot`
// reading `window._APP.devMode` at module-load time; `vi.hoisted` runs before
// the static import below so the global exists in time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
}));

// `ToastList` reads from base-ui's toast manager, so it must live under a
// `Toast.Provider`. A small controller exposes the manager so tests can push
// toasts imperatively.
let manager: ReturnType<typeof Toast.useToastManager> | null = null;

function ManagerBridge() {
  const current = Toast.useToastManager();
  useEffect(() => {
    manager = current;
  });
  return null;
}

function wrap(children: ReactNode) {
  return (
    <Toast.Provider>
      <ManagerBridge />
      {children}
    </Toast.Provider>
  );
}

describe('ToastList', () => {
  it('renders an empty viewport when there are no toasts', () => {
    renderWithProviders(wrap(<ToastList />));

    // No toast title text should be present yet.
    expect(screen.queryByText('hello world')).not.toBeInTheDocument();
  });

  it('renders a toast pushed through the manager', () => {
    renderWithProviders(wrap(<ToastList />));

    act(() => {
      manager?.add({ title: 'hello world', type: 'info' });
    });

    expect(screen.getByText('hello world')).toBeInTheDocument();
    expect(screen.getByTestId('icon-info')).toBeInTheDocument();
  });

  it('shows an error toast with its close control', () => {
    renderWithProviders(wrap(<ToastList />));

    act(() => {
      manager?.add({ title: 'boom', type: 'error' });
    });

    expect(screen.getByText('boom')).toBeInTheDocument();
    expect(screen.getByTestId('icon-error')).toBeInTheDocument();
    // The close control renders with a translated aria-label (it is
    // aria-hidden — and thus outside the a11y tree — until the stack expands).
    expect(screen.getByLabelText('Close')).toBeInTheDocument();
  });

  it('dismisses a toast when closed through the manager', async () => {
    renderWithProviders(wrap(<ToastList />));

    let id = '';
    act(() => {
      id = manager?.add({ title: 'dismiss me', type: 'info' }) ?? '';
    });
    expect(screen.getByText('dismiss me')).toBeInTheDocument();

    act(() => {
      manager?.close(id);
    });

    await waitFor(() =>
      expect(screen.queryByText('dismiss me')).not.toBeInTheDocument(),
    );
  });
});
