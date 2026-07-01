import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { renderWithProviders, seedStore } from '../../../test/harness';
import DataAndPrivacy from './DataAndPrivacy';

// The screen pulls UI from the `../../../ui` barrel, which loads `DaemonDot`
// reading `window._APP.devMode` and calls the Tauri OS plugin's `type()` at
// module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const navigate = vi.fn();
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useNavigate: () => navigate };
});

afterEach(() => {
  navigate.mockReset();
  seedStore({ monitoring: false, networkStats: false });
});

describe('DataAndPrivacy', () => {
  it('renders the logs and diagnostic navigation entries', () => {
    renderWithProviders(<DataAndPrivacy />);

    expect(screen.getByText('Logs (stored locally)')).toBeInTheDocument();
    expect(screen.getByText('Diagnostic tool')).toBeInTheDocument();
  });

  it('reflects the stored switch states', () => {
    seedStore({ networkStats: true, monitoring: false });
    renderWithProviders(<DataAndPrivacy />);

    const switches = screen.getAllByRole('switch');
    expect(switches[0]).toHaveAttribute('aria-checked', 'true');
    expect(switches[1]).toHaveAttribute('aria-checked', 'false');
  });

  it('enables network stats and invokes enable_netstats', async () => {
    const user = userEvent.setup();
    const calls: string[] = [];
    mockIPC((cmd) => {
      calls.push(cmd);
      return undefined;
    });
    seedStore({ networkStats: false });
    renderWithProviders(<DataAndPrivacy />);

    await user.click(screen.getAllByRole('switch')[0]);

    expect(calls).toContain('enable_netstats');
  });

  it('enables error monitoring and invokes enable_sentry', async () => {
    const user = userEvent.setup();
    const calls: string[] = [];
    mockIPC((cmd) => {
      calls.push(cmd);
      return undefined;
    });
    seedStore({ monitoring: false });
    renderWithProviders(<DataAndPrivacy />);

    await user.click(screen.getAllByRole('switch')[1]);

    expect(calls).toContain('enable_sentry');
  });

  it('navigates to the diagnostic screen when its entry is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DataAndPrivacy />);

    await user.click(screen.getByText('Diagnostic tool'));

    expect(navigate).toHaveBeenCalledOnce();
  });
});
