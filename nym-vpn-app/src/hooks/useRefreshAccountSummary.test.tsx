import { describe, expect, it, vi } from 'vitest';
import { act, renderHook, waitFor } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import useRefreshAccountSummary from './useRefreshAccountSummary';

describe('useRefreshAccountSummary', () => {
  it('invokes refresh_account_state with force=true by default', async () => {
    const calls: { cmd: string; payload?: unknown }[] = [];
    mockIPC((cmd, payload) => {
      calls.push({ cmd, payload });
      return undefined;
    });
    const { result } = renderHook(() => useRefreshAccountSummary());

    await act(async () => {
      await result.current.refresh();
    });

    expect(calls).toContainEqual({
      cmd: 'refresh_account_state',
      payload: { force: true },
    });
  });

  it('forwards an explicit force flag to the daemon', async () => {
    const calls: { cmd: string; payload?: unknown }[] = [];
    mockIPC((cmd, payload) => {
      calls.push({ cmd, payload });
      return undefined;
    });
    const { result } = renderHook(() => useRefreshAccountSummary());

    await act(async () => {
      await result.current.refresh(false);
    });

    expect(calls).toContainEqual({
      cmd: 'refresh_account_state',
      payload: { force: false },
    });
  });

  it('clears the refreshing flag once the invocation settles', async () => {
    mockIPC(() => undefined);
    const { result } = renderHook(() => useRefreshAccountSummary());

    await act(async () => {
      await result.current.refresh();
    });

    expect(result.current.refreshing).toBe(false);
  });

  it('flips refreshing to true while an invocation is in flight', async () => {
    let resolveInvoke: (() => void) | undefined;
    mockIPC(
      () =>
        new Promise<undefined>((resolve) => {
          resolveInvoke = () => resolve(undefined);
        }),
    );
    const { result } = renderHook(() => useRefreshAccountSummary());

    let pending: Promise<void> = Promise.resolve();
    act(() => {
      pending = result.current.refresh();
    });

    await waitFor(() => expect(result.current.refreshing).toBe(true));

    await act(async () => {
      resolveInvoke?.();
      await pending;
    });

    expect(result.current.refreshing).toBe(false);
  });

  it('ignores an overlapping refresh while one is already in flight', async () => {
    const spy = vi.fn();
    let resolveInvoke: (() => void) | undefined;
    mockIPC((cmd) => {
      spy(cmd);
      return new Promise<undefined>((resolve) => {
        resolveInvoke = () => resolve(undefined);
      });
    });
    const { result } = renderHook(() => useRefreshAccountSummary());

    let first: Promise<void> = Promise.resolve();
    act(() => {
      first = result.current.refresh();
    });
    await act(async () => {
      await result.current.refresh();
    });

    expect(spy).toHaveBeenCalledOnce();

    await act(async () => {
      resolveInvoke?.();
      await first;
    });
  });
});
