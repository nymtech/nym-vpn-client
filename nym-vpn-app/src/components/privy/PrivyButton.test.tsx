import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useEffect } from 'react';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Toast } from '@base-ui/react';

import { DeeplinkTimeout } from '../../errors';
import { mockTauriCommands, renderWithProviders } from '../../test/harness';
import PrivyButton from './PrivyButton';

// `PrivyButton` pulls from the `../../ui` barrel, which loads `DaemonDot`
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

const openUrl = vi.fn();
const startListening = vi.fn<(timeoutMs?: number) => Promise<string>>();

vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: (url: string): void => {
    openUrl(url);
  },
}));

vi.mock('../../hooks/useDeepLink', () => ({
  default: (): { startListening: typeof startListening } => ({
    startListening,
  }),
}));

function wrap(children: ReactNode) {
  return <Toast.Provider>{children}</Toast.Provider>;
}

describe('PrivyButton', () => {
  beforeEach(() => {
    openUrl.mockReset();
    startListening.mockReset();
    // `get_deep_link` returns the login URL; other commands resolve to null.
    mockTauriCommands((cmd) =>
      cmd === 'get_deep_link' ? 'https://login.example' : null,
    );
  });

  it('renders the label and the external-link icon', () => {
    renderWithProviders(wrap(<PrivyButton label="Continue with Privy" />));

    expect(screen.getByText('Continue with Privy')).toBeInTheDocument();
    expect(screen.getByTestId('icon-open_in_new')).toBeInTheDocument();
  });

  it('fetches the deep link and opens it on click', async () => {
    const user = userEvent.setup();
    // Never resolves within the test so we stay in the pending branch.
    startListening.mockReturnValue(new Promise<string>(() => undefined));

    renderWithProviders(wrap(<PrivyButton label="Login" />));

    await user.click(screen.getByRole('button', { name: /Login/ }));

    await waitFor(() =>
      expect(openUrl).toHaveBeenCalledWith('https://login.example'),
    );
    expect(startListening).toHaveBeenCalledOnce();
  });

  it('shows a timeout toast when the deep link listener times out', async () => {
    const user = userEvent.setup();
    startListening.mockRejectedValue(new DeeplinkTimeout());

    const spy: { titles: ReactNode[] } = { titles: [] };
    function ToastSpy() {
      const manager = Toast.useToastManager();
      useEffect(() => {
        spy.titles = manager.toasts.map((toast) => toast.title);
      });
      return null;
    }

    renderWithProviders(
      wrap(
        <>
          <ToastSpy />
          <PrivyButton label="Login" />
        </>,
      ),
    );

    await user.click(screen.getByRole('button', { name: /Login/ }));

    await waitFor(() => expect(spy.titles).toContain('Login timed out'));
  });
});
