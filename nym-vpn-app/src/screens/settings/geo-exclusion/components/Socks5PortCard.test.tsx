import { describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Toast } from '@base-ui/react';
import { renderWithProviders } from '../../../../test/harness';
import Socks5PortCard from './Socks5PortCard';

// `Socks5PortCard` pulls UI from the `../../../../ui` barrel, which loads
// `DaemonDot` reading `window._APP.devMode` and calls the Tauri OS plugin's
// `type()` at module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('Socks5PortCard', () => {
  it('renders the current listen port and the valid range hint', () => {
    renderWithProviders(
      <Toast.Provider>
        <Socks5PortCard listenPort={1080} onCommitPort={vi.fn()} />
      </Toast.Provider>,
    );

    expect(screen.getByText('SOCKS5 port')).toBeInTheDocument();
    expect(screen.getByText('1024–65535')).toBeInTheDocument();
    expect(screen.getByRole('textbox')).toHaveValue('1080');
  });

  it('shows an error and skips committing for an out-of-range port', async () => {
    const user = userEvent.setup();
    const onCommitPort = vi.fn();
    renderWithProviders(
      <Toast.Provider>
        <Socks5PortCard listenPort={1080} onCommitPort={onCommitPort} />
      </Toast.Provider>,
    );

    const input = screen.getByRole('textbox');
    await user.clear(input);
    await user.type(input, '80');

    expect(screen.getByText(/Invalid port/i)).toBeInTheDocument();
    expect(onCommitPort).not.toHaveBeenCalled();
  });

  it('commits a valid port after the debounce elapses', async () => {
    const user = userEvent.setup();
    const onCommitPort = vi.fn();
    renderWithProviders(
      <Toast.Provider>
        <Socks5PortCard listenPort={1080} onCommitPort={onCommitPort} />
      </Toast.Provider>,
    );

    const input = screen.getByRole('textbox');
    await user.clear(input);
    await user.type(input, '2020');

    // The commit is debounced (~250ms), so it fires after the input settles.
    await waitFor(() =>
      expect(onCommitPort).toHaveBeenCalledExactlyOnceWith(2020),
    );
  });
});
