import { afterEach, describe, expect, it, vi } from 'vitest';
import { act, waitFor } from '@testing-library/react';
import { useAppStore } from '../store';
import { renderHookWithProviders, seedStore } from '../test/harness';
import useAutostart from './useAutostart';

const isEnabled = vi.fn<() => Promise<boolean>>();
const enable = vi.fn<() => Promise<void>>();
const disable = vi.fn<() => Promise<void>>();

vi.mock('@tauri-apps/plugin-autostart', () => ({
  isEnabled: () => isEnabled(),
  enable: () => enable(),
  disable: () => disable(),
}));

afterEach(() => {
  isEnabled.mockReset();
  enable.mockReset();
  disable.mockReset();
  seedStore({ autostart: false });
});

describe('useAutostart', () => {
  it('syncs the store with the plugin state on mount', async () => {
    isEnabled.mockResolvedValue(true);
    seedStore({ autostart: false });

    renderHookWithProviders(() => useAutostart());

    await waitFor(() => expect(useAppStore.getState().autostart).toBe(true));
    expect(isEnabled).toHaveBeenCalled();
  });

  it('exposes the current autostart flag from the store', () => {
    isEnabled.mockResolvedValue(false);
    seedStore({ autostart: true });

    const { result } = renderHookWithProviders(() => useAutostart());

    expect(result.current.enabled).toBe(true);
  });

  it('enables autostart via the plugin when currently disabled', async () => {
    isEnabled.mockResolvedValue(false);
    enable.mockResolvedValue();
    const { result } = renderHookWithProviders(() => useAutostart());

    await act(async () => {
      await result.current.toggle();
    });

    expect(enable).toHaveBeenCalledOnce();
    expect(disable).not.toHaveBeenCalled();
    expect(useAppStore.getState().autostart).toBe(true);
  });

  it('disables autostart via the plugin when currently enabled', async () => {
    isEnabled.mockResolvedValue(true);
    disable.mockResolvedValue();
    const { result } = renderHookWithProviders(() => useAutostart());

    await act(async () => {
      await result.current.toggle();
    });

    expect(disable).toHaveBeenCalledOnce();
    expect(enable).not.toHaveBeenCalled();
    expect(useAppStore.getState().autostart).toBe(false);
  });
});
