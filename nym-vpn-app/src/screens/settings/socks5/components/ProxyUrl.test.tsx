import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ProxyUrl from './ProxyUrl';

// `ProxyUrl` pulls `ButtonIconNew` from the `../../../../ui` barrel, which loads
// modules reading `window._APP.devMode` and calling the Tauri OS plugin's
// `type()` at module-load time. `vi.hoisted`/`vi.mock` run before the imports.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const copy = vi.fn();

// `ProxyUrl` pulls `useClipboard` from the `../../../../hooks` barrel; stub it so
// the toast manager provider isn't required and `copy` calls can be asserted.
vi.mock('../../../../hooks', () => ({
  useClipboard: () => ({ copy }),
}));

describe('ProxyUrl', () => {
  it('renders the title and value', () => {
    render(<ProxyUrl title="SOCKS5 URL" value="socks5h://127.0.0.1:1080" />);

    expect(screen.getByText('SOCKS5 URL')).toBeInTheDocument();
    expect(screen.getByText('socks5h://127.0.0.1:1080')).toBeInTheDocument();
  });

  it('copies the value without a notification when the copy button is clicked', async () => {
    render(<ProxyUrl title="SOCKS5 URL" value="socks5h://127.0.0.1:1080" />);

    await userEvent.click(screen.getByRole('button'));

    expect(copy).toHaveBeenCalledExactlyOnceWith(
      'socks5h://127.0.0.1:1080',
      false,
    );
  });
});
