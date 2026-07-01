import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../test/harness';
import NetworkUpdateDialog from './NetworkUpdateDialog';

// `Dialog` loads the `../../ui` barrel (`DaemonDot` reads `window._APP.devMode`
// at module-load time) and the component itself calls the OS plugin's `type()`.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const openUrl = vi.fn<(url: string) => void>();
vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: (url: string) => openUrl(url),
}));

describe('NetworkUpdateDialog', () => {
  it('renders the update prompt when open', () => {
    renderWithProviders(
      <NetworkUpdateDialog
        isOpen
        onClose={vi.fn()}
        appUpdate
        daemonUpdate={false}
      />,
    );

    expect(screen.getByTestId('update-dialog-title')).toHaveTextContent(
      'Update required!',
    );
    expect(screen.getByTestId('update-dialog-description')).toHaveTextContent(
      'Your app is no longer supported.',
    );
  });

  it('does not render its content when closed', () => {
    renderWithProviders(
      <NetworkUpdateDialog
        isOpen={false}
        onClose={vi.fn()}
        appUpdate
        daemonUpdate
      />,
    );

    expect(screen.queryByTestId('update-dialog-title')).not.toBeInTheDocument();
  });

  it('shows the combined message when both app and daemon are outdated', () => {
    renderWithProviders(
      <NetworkUpdateDialog isOpen onClose={vi.fn()} appUpdate daemonUpdate />,
    );

    expect(screen.getByTestId('update-dialog-description')).toHaveTextContent(
      'Your app and daemon are no longer supported.',
    );
  });

  it('opens the download page and calls onClose on the update button', async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderWithProviders(
      <NetworkUpdateDialog
        isOpen
        onClose={onClose}
        appUpdate
        daemonUpdate={false}
      />,
    );

    await user.click(screen.getByRole('button', { name: 'Update' }));

    expect(openUrl).toHaveBeenCalledWith('https://nym.com/download/linux');
    expect(onClose).toHaveBeenCalledOnce();
  });
});
