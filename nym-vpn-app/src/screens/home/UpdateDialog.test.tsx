import type { ReactElement } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { Toast } from '@base-ui/react';
import { initialState } from '../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../test/harness';
import UpdateDialog from './UpdateDialog';

// `UpdateDialog` captures `window._APP.updaterEnabled` and the OS type at
// module-load time, so both must exist before the static import above runs.
// `devMode` is also read through the `../../ui` barrel (`DaemonDot`). The
// updater is enabled here so the component renders instead of short-circuiting
// to `null`.
vi.hoisted(() => {
  (
    globalThis as unknown as {
      _APP: { devMode: boolean; updaterEnabled: boolean };
    }
  )._APP = {
    devMode: false,
    updaterEnabled: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

vi.mock('@tauri-apps/plugin-process', () => ({
  relaunch: vi.fn(),
}));

// `useToast` reads the base-ui toast manager, which requires a `Toast.Provider`.
function renderDialog(ui: ReactElement) {
  return renderWithProviders(<Toast.Provider>{ui}</Toast.Provider>);
}

afterEach(() => {
  seedStore({ ...initialState });
});

describe('UpdateDialog', () => {
  it('mounts closed and shows no dialog content by default', () => {
    renderDialog(<UpdateDialog />);

    // On Linux the dialog stays closed until an update is fetched, so no title
    // should be visible on first render.
    expect(screen.queryByTestId('update-dialog-title')).not.toBeInTheDocument();
  });

  it('does not render a visible dialog panel while closed', () => {
    renderDialog(<UpdateDialog />);

    expect(screen.queryByTestId('update-dialog')).not.toBeInTheDocument();
  });

  it('mounts without error when a Linux app update landed', () => {
    // Exercises the linux restart-toast effect (which calls the toast manager)
    // without asserting on toast rendering, which lives in a separate viewport.
    seedStore({ linuxAppUpdated: true });

    renderDialog(<UpdateDialog />);

    expect(screen.queryByTestId('update-dialog-title')).not.toBeInTheDocument();
  });
});
