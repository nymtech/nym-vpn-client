import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { DeeplinkTimeout } from '../errors';
import useDeepLink from './useDeepLink';

// Controllable mock of the deep-link plugin. `onOpenUrl` registers a handler and
// resolves with an unlisten fn; the test drives the handler via `emit`.
type Handler = (urls: string[]) => void;
let capturedHandler: Handler | null = null;
const unlisten = vi.fn();
let registrationError: Error | null = null;

const onOpenUrl = vi.fn((handler: Handler): Promise<typeof unlisten> => {
  capturedHandler = handler;
  if (registrationError) return Promise.reject(registrationError);
  return Promise.resolve(unlisten);
});

vi.mock('@tauri-apps/plugin-deep-link', () => ({
  onOpenUrl: (handler: Handler) => onOpenUrl(handler),
}));

function emit(urls: string[]) {
  capturedHandler?.(urls);
}

beforeEach(() => {
  capturedHandler = null;
  registrationError = null;
  onOpenUrl.mockClear();
  unlisten.mockClear();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('useDeepLink', () => {
  it('resolves with the first URL delivered to the listener', async () => {
    const { result } = renderHook(() => useDeepLink());

    let promise: Promise<string> = Promise.resolve('');
    await act(async () => {
      promise = result.current.startListening();
      // let onOpenUrl resolve so the unlisten fn is stored
      await Promise.resolve();
    });

    await act(async () => {
      emit(['nym://link', 'nym://ignored']);
      await Promise.resolve();
    });

    await expect(promise).resolves.toBe('nym://link');
    // the listener is torn down after a successful resolution
    expect(unlisten).toHaveBeenCalledOnce();
  });

  it('ignores empty url batches until a real url arrives', async () => {
    const { result } = renderHook(() => useDeepLink());

    let promise: Promise<string> = Promise.resolve('');
    await act(async () => {
      promise = result.current.startListening();
      await Promise.resolve();
    });

    await act(async () => {
      emit([]);
      emit(['nym://real']);
      await Promise.resolve();
    });

    await expect(promise).resolves.toBe('nym://real');
  });

  it('rejects with DeeplinkTimeout when no url arrives before the deadline', async () => {
    vi.useFakeTimers();
    const { result } = renderHook(() => useDeepLink());

    let promise: Promise<string> = Promise.resolve('');
    act(() => {
      promise = result.current.startListening(1000);
    });
    // flush the onOpenUrl microtask registration
    await vi.advanceTimersByTimeAsync(0);

    const assertion = expect(promise).rejects.toBeInstanceOf(DeeplinkTimeout);
    await vi.advanceTimersByTimeAsync(1000);
    await assertion;
  });

  it('rejects when registering the listener fails', async () => {
    registrationError = new Error('register failed');
    const { result } = renderHook(() => useDeepLink());

    let promise: Promise<string> = Promise.resolve('');
    act(() => {
      promise = result.current.startListening();
    });

    await expect(promise).rejects.toThrow('register failed');
  });

  it('cleans up the listener on unmount', async () => {
    const { result, unmount } = renderHook(() => useDeepLink());

    await act(async () => {
      void result.current.startListening();
      await Promise.resolve();
    });

    unmount();
    expect(unlisten).toHaveBeenCalledOnce();
  });
});
