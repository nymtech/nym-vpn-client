import { beforeEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockTauriCommands, renderWithProviders } from '../../../test/harness';
import ActionMenu from './ActionMenu';

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

const add = vi.fn();
vi.mock('../../../hooks', () => ({
  useToast: () => ({ add, close: vi.fn() }),
}));

beforeEach(() => {
  add.mockReset();
});

async function openMenu() {
  const user = userEvent.setup();
  await user.click(screen.getByText('more_vert'));
  return user;
}

describe('ActionMenu', () => {
  it('renders a menu trigger without any dialog open initially', () => {
    renderWithProviders(<ActionMenu />);

    expect(screen.getByText('more_vert')).toBeInTheDocument();
    expect(screen.queryByText('Delete NymVPN logs?')).not.toBeInTheDocument();
  });

  it('opens the delete confirmation dialog from the menu', async () => {
    renderWithProviders(<ActionMenu />);

    const user = await openMenu();
    await user.click(await screen.findByText('Delete'));

    expect(await screen.findByText('Delete NymVPN logs?')).toBeInTheDocument();
  });

  it('deletes the logs and shows a success toast on confirm', async () => {
    const commands: string[] = [];
    mockTauriCommands((cmd) => {
      commands.push(cmd);
      return undefined;
    });
    renderWithProviders(<ActionMenu />);

    const user = await openMenu();
    await user.click(await screen.findByText('Delete'));
    await user.click(await screen.findByText('Delete logs'));

    await waitFor(() =>
      expect(add).toHaveBeenCalledWith({
        title: 'Logs deleted successfully',
        type: 'info',
      }),
    );
    expect(commands).toContain('delete_logs');
    expect(commands).toContain('delete_app_logs');
  });

  it('shows an error toast when exporting the logs fails', async () => {
    mockTauriCommands((cmd) => {
      if (cmd === 'zip_logs') throw new Error('boom');
      return undefined;
    });
    renderWithProviders(<ActionMenu />);

    const user = await openMenu();
    await user.click(await screen.findByText('Export as ZIP'));
    await user.click(await screen.findByText('Export anyway'));

    await waitFor(() =>
      expect(add).toHaveBeenCalledWith({
        title: 'Failed to export logs',
        type: 'error',
      }),
    );
  });
});
