import { beforeEach, describe, expect, it, vi } from 'vitest';
import { act, waitFor } from '@testing-library/react';
import {
  mockTauriCommands,
  renderHookWithProviders,
  seedStore,
} from '../test/harness';
import { useAppStore } from '../store';
import useLogout from './useLogout';

const noop = () => undefined;

// A daemon rejection carries a backend error key; model it as a real Error so
// nothing throws a bare object.
class BackendErr extends Error {
  key: string;
  constructor(key: string) {
    super(key);
    this.name = 'BackendErr';
    this.key = key;
  }
}

// `useToast` needs a base-ui Toast.Provider that the harness does not supply;
// mock the hooks barrel so the toast surface is observable without it.
const add = vi.fn<(data: { type: string; title: string }) => void>();
vi.mock('./index', () => ({
  useToast: () => ({ add, close: noop }),
}));

// Cache deletions hit the kv store over IPC; stub them out.
vi.mock('../cache', () => ({
  CCache: { del: vi.fn().mockResolvedValue(undefined) },
}));

function install() {
  const calls: string[] = [];
  mockTauriCommands((cmd) => {
    calls.push(cmd);
    return undefined;
  });
  return calls;
}

beforeEach(() => {
  add.mockReset();
  seedStore({ state: 'disconnected' });
});

describe('useLogout', () => {
  it('logs out directly when already disconnected', async () => {
    seedStore({ state: 'disconnected' });
    const calls = install();
    const { result } = renderHookWithProviders(() => useLogout());

    await act(async () => {
      await result.current.logout();
    });

    expect(calls).toContain('forget_account');
    expect(add).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'success' }),
    );
    expect(useAppStore.getState().account).toBe(false);
    expect(result.current.loading).toBe(false);
  });

  it('disconnects first, then logs out once the tunnel is down', async () => {
    seedStore({ state: 'connected' });
    const calls = install();
    const { result } = renderHookWithProviders(() => useLogout());

    await act(async () => {
      await result.current.logout();
    });

    // disconnect was requested, but forget_account is deferred to the effect
    expect(calls).toContain('disconnect');
    expect(calls).not.toContain('forget_account');
    expect(result.current.loading).toBe(true);

    // the daemon reports the tunnel down -> effect completes the logout
    await act(async () => {
      seedStore({ state: 'disconnected' });
      await Promise.resolve();
    });

    await waitFor(() => expect(calls).toContain('forget_account'));
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(add).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'success' }),
    );
  });

  it('ignores a second logout while one is in flight', async () => {
    seedStore({ state: 'connected' });
    const calls = install();
    const { result } = renderHookWithProviders(() => useLogout());

    await act(async () => {
      await result.current.logout();
      await result.current.logout();
    });

    // only one disconnect issued despite two calls
    expect(calls.filter((c) => c === 'disconnect')).toHaveLength(1);
  });

  it('surfaces an error toast when forget_account fails', async () => {
    seedStore({ state: 'disconnected' });
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(noop);
    mockTauriCommands((cmd) => {
      if (cmd === 'forget_account') throw new BackendErr('internal');
      return undefined;
    });
    const { result } = renderHookWithProviders(() => useLogout());

    await act(async () => {
      await result.current.logout();
    });

    expect(add).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'error' }),
    );
    expect(result.current.loading).toBe(false);
    errorSpy.mockRestore();
  });

  it('recovers when the disconnect call itself fails', async () => {
    seedStore({ state: 'connected' });
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(noop);
    mockTauriCommands((cmd) => {
      if (cmd === 'disconnect') throw new BackendErr('internal');
      return undefined;
    });
    const { result } = renderHookWithProviders(() => useLogout());

    await act(async () => {
      await result.current.logout();
    });

    expect(add).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'error' }),
    );
    await waitFor(() => expect(result.current.loading).toBe(false));
    errorSpy.mockRestore();
  });
});
