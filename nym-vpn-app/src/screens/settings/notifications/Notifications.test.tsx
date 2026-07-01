import { describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  mockTauriCommands,
  renderWithProviders,
  seedStore,
} from '../../../test/harness';
import { useAppStore } from '../../../store';
import Notifications from './Notifications';

// The `../../../ui` barrel loads `DaemonDot`, which reads `window._APP.devMode`
// at module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const toggleDesktopNotifications = vi.fn();
vi.mock('../../../hooks/index', () => ({
  useDesktopNotifications: () => toggleDesktopNotifications,
}));

describe('Notifications', () => {
  it('renders both notification toggles', () => {
    seedStore({
      desktopNotifications: false,
      gatewayIndependenceNotifications: false,
    });
    renderWithProviders(<Notifications />);

    expect(screen.getByText('Server family reminders')).toBeInTheDocument();
    expect(screen.getByText('Desktop notifications')).toBeInTheDocument();
    expect(screen.getAllByRole('switch')).toHaveLength(2);
  });

  it('reflects the current store state in the switches', () => {
    seedStore({
      desktopNotifications: true,
      gatewayIndependenceNotifications: false,
    });
    renderWithProviders(<Notifications />);

    const [family, desktop] = screen.getAllByRole('switch');
    expect(family).toHaveAttribute('aria-checked', 'false');
    expect(desktop).toHaveAttribute('aria-checked', 'true');
  });

  it('optimistically toggles family reminders and calls the backend', async () => {
    seedStore({
      desktopNotifications: false,
      gatewayIndependenceNotifications: false,
    });
    const commands: string[] = [];
    mockTauriCommands((cmd) => {
      commands.push(cmd);
      return undefined;
    });
    renderWithProviders(<Notifications />);

    const [family] = screen.getAllByRole('switch');
    await userEvent.click(family);

    await waitFor(() =>
      expect(commands).toContain('set_gateway_independence_notifications'),
    );
    expect(useAppStore.getState().gatewayIndependenceNotifications).toBe(true);
  });

  it('reverts the optimistic family-reminder update when the backend fails', async () => {
    seedStore({
      desktopNotifications: false,
      gatewayIndependenceNotifications: false,
    });
    mockTauriCommands((cmd) => {
      if (cmd === 'set_gateway_independence_notifications') {
        throw new Error('backend down');
      }
      return undefined;
    });
    renderWithProviders(<Notifications />);

    const [family] = screen.getAllByRole('switch');
    await userEvent.click(family);

    await waitFor(() =>
      expect(useAppStore.getState().gatewayIndependenceNotifications).toBe(
        false,
      ),
    );
  });

  it('delegates desktop notifications toggling to the hook', async () => {
    seedStore({
      desktopNotifications: false,
      gatewayIndependenceNotifications: false,
    });
    renderWithProviders(<Notifications />);

    const [, desktop] = screen.getAllByRole('switch');
    await userEvent.click(desktop);

    expect(toggleDesktopNotifications).toHaveBeenCalled();
  });
});
