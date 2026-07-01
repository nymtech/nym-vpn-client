import { beforeEach, describe, expect, it, vi } from 'vitest';
import { act, waitFor } from '@testing-library/react';
import { renderHookWithProviders, seedStore } from '../test/harness';
import { useAppStore } from '../store';
import useDesktopNotifications from './useDesktopNotifications';

const isPermissionGranted = vi.fn<() => Promise<boolean>>();
const requestPermission =
  vi.fn<() => Promise<'granted' | 'denied' | 'default'>>();
vi.mock('@tauri-apps/plugin-notification', () => ({
  isPermissionGranted: () => isPermissionGranted(),
  requestPermission: () => requestPermission(),
}));

const kvSet = vi.fn<(key: string, value: unknown) => void>();
vi.mock('../kvStore', () => ({
  kvSet: (key: string, value: unknown) => {
    kvSet(key, value);
  },
}));

beforeEach(() => {
  isPermissionGranted.mockReset();
  requestPermission.mockReset();
  kvSet.mockReset();
});

describe('useDesktopNotifications mount effect', () => {
  it('requests OS permission when enabled but not yet granted', async () => {
    seedStore({ desktopNotifications: true });
    isPermissionGranted.mockResolvedValue(false);
    requestPermission.mockResolvedValue('granted');

    renderHookWithProviders(() => useDesktopNotifications());

    await waitFor(() => expect(requestPermission).toHaveBeenCalledOnce());
    expect(useAppStore.getState().desktopNotifications).toBe(true);
    expect(kvSet).toHaveBeenCalledWith('desktop-notifications', true);
  });

  it('disables notifications in the store when permission is denied', async () => {
    seedStore({ desktopNotifications: true });
    isPermissionGranted.mockResolvedValue(false);
    requestPermission.mockResolvedValue('denied');

    renderHookWithProviders(() => useDesktopNotifications());

    await waitFor(() =>
      expect(useAppStore.getState().desktopNotifications).toBe(false),
    );
    expect(kvSet).toHaveBeenCalledWith('desktop-notifications', false);
  });

  it('does not prompt when permission is already granted', async () => {
    seedStore({ desktopNotifications: true });
    isPermissionGranted.mockResolvedValue(true);

    renderHookWithProviders(() => useDesktopNotifications());

    await waitFor(() => expect(isPermissionGranted).toHaveBeenCalled());
    expect(requestPermission).not.toHaveBeenCalled();
  });

  it('does not prompt when notifications are disabled', async () => {
    seedStore({ desktopNotifications: false });
    isPermissionGranted.mockResolvedValue(false);

    renderHookWithProviders(() => useDesktopNotifications());

    await waitFor(() => expect(isPermissionGranted).toHaveBeenCalled());
    expect(requestPermission).not.toHaveBeenCalled();
  });
});

describe('useDesktopNotifications toggle', () => {
  it('turns notifications ON, requesting permission first', async () => {
    seedStore({ desktopNotifications: false });
    isPermissionGranted.mockResolvedValue(false);
    requestPermission.mockResolvedValue('granted');

    const { result } = renderHookWithProviders(() => useDesktopNotifications());

    await act(async () => {
      await result.current();
    });

    expect(requestPermission).toHaveBeenCalled();
    expect(useAppStore.getState().desktopNotifications).toBe(true);
    expect(kvSet).toHaveBeenCalledWith('desktop-notifications', true);
  });

  it('leaves notifications OFF when the user denies permission', async () => {
    seedStore({ desktopNotifications: false });
    isPermissionGranted.mockResolvedValue(false);
    requestPermission.mockResolvedValue('denied');

    const { result } = renderHookWithProviders(() => useDesktopNotifications());

    await act(async () => {
      await result.current();
    });

    // enabled stays equal to the previous value (false) -> no persistence
    expect(useAppStore.getState().desktopNotifications).toBe(false);
    expect(kvSet).not.toHaveBeenCalled();
  });

  it('turns notifications OFF without prompting for permission', async () => {
    seedStore({ desktopNotifications: true });
    isPermissionGranted.mockResolvedValue(true);

    const { result } = renderHookWithProviders(() => useDesktopNotifications());

    await act(async () => {
      await result.current();
    });

    expect(requestPermission).not.toHaveBeenCalled();
    expect(useAppStore.getState().desktopNotifications).toBe(false);
    expect(kvSet).toHaveBeenCalledWith('desktop-notifications', false);
  });
});
