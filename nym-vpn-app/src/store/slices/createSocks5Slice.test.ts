import { beforeEach, describe, expect, it, vi } from 'vitest';
import { type StateCreator, create } from 'zustand';
import { mockIPC } from '@tauri-apps/api/mocks';
import type {
  HttpRpcSettings,
  SelectedNode,
  Socks5Settings,
  Socks5Status,
} from '../../types';
import { type Socks5Slice, createSocks5Slice } from './createSocks5Slice';

const makeStore = () =>
  create<Socks5Slice>()(
    createSocks5Slice as unknown as StateCreator<Socks5Slice>,
  );

let store: ReturnType<typeof makeStore>;
const status = { enabled: true } as unknown as Socks5Status;
const socks5Settings = {} as Socks5Settings;
const httpRpcSettings = {} as HttpRpcSettings;
const exit = {} as SelectedNode;

beforeEach(() => {
  store = makeStore();
});

describe('createSocks5Slice initial state', () => {
  it('starts idle with no status', () => {
    expect(store.getState().status).toBeNull();
    expect(store.getState().isLoading).toBe(false);
  });
});

describe('refresh', () => {
  it('stores the status returned by the daemon', async () => {
    mockIPC((cmd) => (cmd === 'get_socks5_status' ? status : undefined));
    await store.getState().refresh();
    expect(store.getState().status).toEqual(status);
  });

  it('silently ignores a failed status query', async () => {
    mockIPC(() => {
      throw new Error('poll failed');
    });
    await store.getState().refresh();
    expect(store.getState().status).toBeNull();
  });
});

describe('enable', () => {
  it('invokes enable_socks5 then refreshes and clears loading', async () => {
    const calls: string[] = [];
    mockIPC((cmd) => {
      calls.push(cmd);
      return cmd === 'get_socks5_status' ? status : undefined;
    });
    await store.getState().enable(socks5Settings, httpRpcSettings, exit);
    expect(calls).toContain('enable_socks5');
    expect(store.getState().status).toEqual(status);
    expect(store.getState().isLoading).toBe(false);
  });

  it('ignores a duplicate call while already loading', async () => {
    store.setState({ isLoading: true });
    const spy = vi.fn();
    mockIPC((cmd) => {
      spy(cmd);
      return undefined;
    });
    await store.getState().enable(socks5Settings, httpRpcSettings, exit);
    expect(spy).not.toHaveBeenCalled();
  });

  it('rethrows on failure but resets loading', async () => {
    mockIPC((cmd) => {
      if (cmd === 'enable_socks5') throw new Error('nope');
      return undefined;
    });
    await expect(
      store.getState().enable(socks5Settings, httpRpcSettings, exit),
    ).rejects.toThrow();
    expect(store.getState().isLoading).toBe(false);
  });
});

describe('disable', () => {
  it('invokes disable_socks5 then refreshes', async () => {
    const calls: string[] = [];
    mockIPC((cmd) => {
      calls.push(cmd);
      return cmd === 'get_socks5_status' ? status : undefined;
    });
    await store.getState().disable();
    expect(calls).toContain('disable_socks5');
    expect(store.getState().isLoading).toBe(false);
  });

  it('rethrows on failure but resets loading', async () => {
    mockIPC((cmd) => {
      if (cmd === 'disable_socks5') throw new Error('nope');
      return undefined;
    });
    await expect(store.getState().disable()).rejects.toThrow();
    expect(store.getState().isLoading).toBe(false);
  });
});
