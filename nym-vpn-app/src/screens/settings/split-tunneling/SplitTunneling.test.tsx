import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../../test/harness';
import type { AppEntry } from './AppItem';
import SplitTunneling from './SplitTunneling';

// `SplitTunneling` calls the Tauri OS plugin's `type()` to branch on platform
// and pulls UI from the barrel that reads `window._APP.devMode` at module-load
// time. `vi.hoisted`/`vi.mock` run before the imports; `osType` is mutable so
// individual tests can exercise the Linux and Windows branches.
const osType = vi.hoisted<{ value: 'linux' | 'windows' }>(() => ({
  value: 'linux',
}));

vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => osType.value,
  platform: () => osType.value,
}));

vi.mock('@tauri-apps/plugin-shell', () => ({
  Command: { create: vi.fn() },
}));

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => path,
  invoke: vi.fn(),
}));

const add = vi.fn();
vi.mock('../../../hooks/index', () => ({
  useToast: () => ({ add, close: vi.fn() }),
}));

// Control the split-tunnel hook so the component can be driven without the
// daemon; keep `parseExecArgs` (also exported from `./utils`) intact.
const splitTunnel = vi.hoisted(() => ({
  apps: [] as AppEntry[],
  enabled: false,
  loading: false,
  isSupported: true,
  setEnabled: vi.fn<() => Promise<void>>(),
  add: vi.fn<() => Promise<void>>(),
  addCustomApp: vi.fn<() => Promise<void>>(),
  remove: vi.fn<() => Promise<void>>(),
  removeCustomApp: vi.fn<() => Promise<void>>(),
}));

vi.mock('./utils', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./utils')>();
  return { ...actual, useSplitTunnel: () => splitTunnel };
});

function makeApp(overrides: Partial<AppEntry> = {}): AppEntry {
  return {
    name: 'Firefox',
    executable_path: '/usr/bin/firefox',
    icon: null,
    is_custom: false,
    state: 'excluded',
    ...overrides,
  };
}

afterEach(() => {
  osType.value = 'linux';
  splitTunnel.apps = [];
  splitTunnel.enabled = false;
  splitTunnel.loading = false;
  splitTunnel.isSupported = true;
  add.mockReset();
  splitTunnel.setEnabled.mockReset();
  splitTunnel.addCustomApp.mockReset();
});

describe('SplitTunneling', () => {
  it('shows a spinner while loading', () => {
    splitTunnel.loading = true;
    renderWithProviders(<SplitTunneling />);

    expect(screen.getByTestId('button-spinner')).toBeInTheDocument();
  });

  it('shows an unsupported message when split tunneling is not supported', () => {
    splitTunnel.isSupported = false;
    renderWithProviders(<SplitTunneling />);

    expect(
      screen.getByText('Split tunneling is not supported on this platform'),
    ).toBeInTheDocument();
  });

  it('renders the empty app list on Linux', () => {
    osType.value = 'linux';
    renderWithProviders(<SplitTunneling />);

    expect(screen.getByText('Apps (0)')).toBeInTheDocument();
    expect(screen.getByText('Exclude custom app')).toBeInTheDocument();
  });

  it('renders the populated app list grouped by letter', () => {
    osType.value = 'linux';
    splitTunnel.apps = [makeApp(), makeApp({ name: 'Slack' })];
    renderWithProviders(<SplitTunneling />);

    expect(screen.getByText('Apps (2)')).toBeInTheDocument();
    expect(screen.getByText('Firefox')).toBeInTheDocument();
    expect(screen.getByText('Slack')).toBeInTheDocument();
  });

  it('adds a custom app when the add card is clicked', async () => {
    splitTunnel.addCustomApp.mockResolvedValue(undefined);
    renderWithProviders(<SplitTunneling />);

    await userEvent.click(screen.getByText('Exclude custom app'));

    expect(splitTunnel.addCustomApp).toHaveBeenCalledOnce();
  });

  it('toggles split tunneling on via the enable switch (Windows)', async () => {
    osType.value = 'windows';
    splitTunnel.setEnabled.mockResolvedValue(undefined);
    renderWithProviders(<SplitTunneling />);

    await userEvent.click(screen.getByText('Enable split tunneling'));

    expect(splitTunnel.setEnabled).toHaveBeenCalledWith(true);
  });
});
