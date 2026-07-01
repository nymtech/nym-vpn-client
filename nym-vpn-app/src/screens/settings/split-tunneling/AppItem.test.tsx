import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../../test/harness';
import AppItem, { type AppEntry } from './AppItem';

// `AppItem` calls the Tauri OS plugin's `type()` to branch on the platform, and
// `MsIcon` (from `../../../ui/MsIcon`) reads `window._APP.devMode` at
// module-load time. `vi.hoisted`/`vi.mock` run before the imports; `osType` is
// mutable so individual tests can exercise the Linux and Windows branches.
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

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => path,
}));

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
});

describe('AppItem', () => {
  it('renders the app name', () => {
    renderWithProviders(<AppItem app={makeApp()} onStateChange={vi.fn()} />);

    expect(screen.getByText('Firefox')).toBeInTheDocument();
  });

  it('launches the app on click (Linux)', async () => {
    osType.value = 'linux';
    const onLaunch = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    renderWithProviders(
      <AppItem app={makeApp()} onStateChange={vi.fn()} onLaunch={onLaunch} />,
    );

    await userEvent.click(screen.getByText('Firefox'));

    expect(onLaunch).toHaveBeenCalledOnce();
  });

  it('renders a remove button for custom apps and calls onRemove', async () => {
    osType.value = 'linux';
    const onRemove = vi.fn();
    renderWithProviders(
      <AppItem
        app={makeApp({ is_custom: true })}
        onStateChange={vi.fn()}
        onLaunch={vi.fn()}
        onRemove={onRemove}
      />,
    );

    await userEvent.click(
      screen.getByRole('button', { name: 'Remove Firefox' }),
    );

    expect(onRemove).toHaveBeenCalledOnce();
  });

  it('marks a problematic (disabled) app', () => {
    renderWithProviders(
      <AppItem
        app={makeApp({
          name: 'gnome-terminal',
          executable_path: '/usr/bin/gnome-terminal',
        })}
        onStateChange={vi.fn()}
      />,
    );

    expect(
      screen.getByText("App isn't available for split tunneling"),
    ).toBeInTheDocument();
  });

  it('renders include/exclude controls and reports state changes (Windows)', async () => {
    osType.value = 'windows';
    const onStateChange = vi
      .fn<() => Promise<void>>()
      .mockResolvedValue(undefined);
    renderWithProviders(
      <AppItem app={makeApp()} onStateChange={onStateChange} />,
    );

    await userEvent.click(
      screen.getByRole('button', { name: 'Exclude Firefox from VPN' }),
    );

    expect(onStateChange).toHaveBeenCalledWith(
      expect.objectContaining({ name: 'Firefox' }),
      'included',
    );
  });
});
