import { beforeEach, describe, expect, it, vi } from 'vitest';
import { act } from '@testing-library/react';
import { AppName } from '../constants';
import { renderHookWithProviders, seedStore } from '../test/harness';
import useNotify from './useNotify';

const isPermissionGranted = vi.fn<() => Promise<boolean>>();
const sendNotification = vi.fn<(arg: unknown) => void>();
vi.mock('@tauri-apps/plugin-notification', () => ({
  isPermissionGranted: () => isPermissionGranted(),
  sendNotification: (arg: unknown) => {
    sendNotification(arg);
  },
}));

const osType = vi.fn<() => string>();
vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => osType(),
}));

const isFocused = vi.fn<() => Promise<boolean>>();
const isVisible = vi.fn<() => Promise<boolean>>();
vi.mock('@tauri-apps/api/webviewWindow', () => ({
  getCurrentWebviewWindow: () => ({
    isFocused: () => isFocused(),
    isVisible: () => isVisible(),
  }),
}));

beforeEach(() => {
  isPermissionGranted.mockReset();
  sendNotification.mockReset();
  osType.mockReset();
  isFocused.mockReset();
  isVisible.mockReset();
  osType.mockReturnValue('windows');
  isPermissionGranted.mockResolvedValue(true);
  // default: window in the background so the anti-focus guard passes
  isFocused.mockResolvedValue(false);
  isVisible.mockResolvedValue(true);
});

describe('useNotify', () => {
  it('sends a plain-body notification off Linux when enabled', async () => {
    seedStore({ desktopNotifications: true });
    const { result } = renderHookWithProviders(() => useNotify());

    await act(async () => {
      await result.current.notify('hello');
    });

    expect(sendNotification).toHaveBeenCalledWith('hello');
  });

  it('sends a titled notification on Linux', async () => {
    seedStore({ desktopNotifications: true });
    osType.mockReturnValue('linux');
    const { result } = renderHookWithProviders(() => useNotify());

    await act(async () => {
      await result.current.notify('hi');
    });

    expect(sendNotification).toHaveBeenCalledWith({
      title: AppName,
      body: 'hi',
    });
  });

  it('does nothing when desktop notifications are disabled', async () => {
    seedStore({ desktopNotifications: false });
    const { result } = renderHookWithProviders(() => useNotify());

    await act(async () => {
      await result.current.notify('nope');
    });

    expect(sendNotification).not.toHaveBeenCalled();
    expect(isPermissionGranted).not.toHaveBeenCalled();
  });

  it('suppresses a notification when the window is focused and visible', async () => {
    seedStore({ desktopNotifications: true });
    isFocused.mockResolvedValue(true);
    isVisible.mockResolvedValue(true);
    const { result } = renderHookWithProviders(() => useNotify());

    await act(async () => {
      await result.current.notify('shh');
    });

    expect(sendNotification).not.toHaveBeenCalled();
  });

  it('still sends while focused when force is set', async () => {
    seedStore({ desktopNotifications: true });
    isFocused.mockResolvedValue(true);
    isVisible.mockResolvedValue(true);
    const { result } = renderHookWithProviders(() => useNotify());

    await act(async () => {
      await result.current.notify('urgent', { force: true });
    });

    expect(sendNotification).toHaveBeenCalledWith('urgent');
  });

  it('deduplicates consecutive identical notifications', async () => {
    seedStore({ desktopNotifications: true });
    const { result } = renderHookWithProviders(() => useNotify());

    await act(async () => {
      await result.current.notify('same');
    });
    await act(async () => {
      await result.current.notify('same');
    });

    expect(sendNotification).toHaveBeenCalledOnce();
  });

  it('bypasses the spam check with noSpamCheck', async () => {
    seedStore({ desktopNotifications: true });
    const { result } = renderHookWithProviders(() => useNotify());

    await act(async () => {
      await result.current.notify('twice', { noSpamCheck: true });
    });
    await act(async () => {
      await result.current.notify('twice', { noSpamCheck: true });
    });

    expect(sendNotification).toHaveBeenCalledTimes(2);
  });

  it('skips sending when the OS permission is not granted', async () => {
    seedStore({ desktopNotifications: true });
    isPermissionGranted.mockResolvedValue(false);
    const { result } = renderHookWithProviders(() => useNotify());

    await act(async () => {
      await result.current.notify('blocked');
    });

    expect(isPermissionGranted).toHaveBeenCalled();
    expect(sendNotification).not.toHaveBeenCalled();
  });

  it('does not suppress when the user is not on the guarded screen', async () => {
    seedStore({ desktopNotifications: true });
    isFocused.mockResolvedValue(true);
    isVisible.mockResolvedValue(true);
    // current route is '/', so guarding '/home' means "not on the right screen"
    const { result } = renderHookWithProviders(() => useNotify(), {
      initialEntries: ['/'],
    });

    await act(async () => {
      await result.current.notify('elsewhere', { locationPath: '/home' });
    });

    expect(sendNotification).toHaveBeenCalledWith('elsewhere');
  });
});
