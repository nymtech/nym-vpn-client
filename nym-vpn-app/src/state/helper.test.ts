import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { VpndInfo, VpndStatus } from '../types';

import { dispatch } from '../store';
import { kvGet, kvSet } from '../kvStore';
import {
  daemonStatusUpdate,
  fireRequests,
  getVpndInfo,
  networkEnvChanged,
} from './helper';

// The store module creates the global Zustand store and pulls in slices; mock it
// so `dispatch` is observable and no real side effects run.
vi.mock('../store', () => ({ dispatch: vi.fn() }));
// kvStore hits Tauri `invoke`; mock it so network-env logic is deterministic.
vi.mock('../kvStore', () => ({ kvGet: vi.fn(), kvSet: vi.fn() }));

const info = (network: string): VpndInfo =>
  ({ network }) as unknown as VpndInfo;

const okStatus = (i: VpndInfo | null): VpndStatus => ({ ok: i });

const nonCompatStatus = (i: VpndInfo): VpndStatus => ({
  nonCompat: { current: i, requirement: '>=1.0.0' },
});

beforeEach(() => {
  vi.clearAllMocks();
});

describe('getVpndInfo', () => {
  it('returns null when daemon is down', () => {
    expect(getVpndInfo('down')).toBeNull();
  });

  it('returns null for an ok status with null info', () => {
    expect(getVpndInfo(okStatus(null))).toBeNull();
  });

  it('returns the info for an ok status', () => {
    const i = info('mainnet');
    expect(getVpndInfo(okStatus(i))).toBe(i);
  });

  it('returns the current info for a non-compat status', () => {
    const i = info('mainnet');
    expect(getVpndInfo(nonCompatStatus(i))).toBe(i);
  });
});

describe('fireRequests', () => {
  it('calls onFulfilled with the resolved value for fulfilled requests', async () => {
    const onFulfilled = vi.fn();
    await fireRequests([
      { name: 'ok', request: () => Promise.resolve(42), onFulfilled },
    ]);
    expect(onFulfilled).toHaveBeenCalledWith(42);
  });

  it('skips onFulfilled for rejected requests', async () => {
    const onFulfilled = vi.fn();
    await fireRequests([
      {
        name: 'boom',
        request: () => Promise.reject(new Error('x')),
        onFulfilled,
      },
    ]);
    expect(onFulfilled).not.toHaveBeenCalled();
  });

  it('handles a mix of fulfilled and rejected requests', async () => {
    const okCb = vi.fn();
    const failCb = vi.fn();
    await fireRequests([
      { name: 'ok', request: () => Promise.resolve('v'), onFulfilled: okCb },
      {
        name: 'fail',
        request: () => Promise.reject(new Error('e')),
        onFulfilled: failCb,
      },
    ]);
    expect(okCb).toHaveBeenCalledWith('v');
    expect(failCb).not.toHaveBeenCalled();
  });
});

describe('networkEnvChanged', () => {
  it('returns false for a down daemon without touching the store', async () => {
    expect(await networkEnvChanged('down')).toBe(false);
    expect(kvGet).not.toHaveBeenCalled();
  });

  it('returns false when auth is denied', async () => {
    expect(await networkEnvChanged('authDenied')).toBe(false);
  });

  it('returns true and persists when the network env changed', async () => {
    vi.mocked(kvGet).mockResolvedValue('mainnet');
    const changed = await networkEnvChanged(okStatus(info('sandbox')));
    expect(changed).toBe(true);
    expect(kvSet).toHaveBeenCalledWith('last-network-env', 'sandbox');
  });

  it('returns false and does not persist when the network env is unchanged', async () => {
    vi.mocked(kvGet).mockResolvedValue('mainnet');
    const changed = await networkEnvChanged(okStatus(info('mainnet')));
    expect(changed).toBe(false);
    expect(kvSet).not.toHaveBeenCalled();
  });
});

describe('daemonStatusUpdate', () => {
  it('dispatches the mapped daemon status', () => {
    daemonStatusUpdate(okStatus(info('mainnet')), vi.fn(), vi.fn());
    expect(dispatch).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'set-daemon-status', status: 'ok' }),
    );
  });

  it('adds an error toast and closes nothing extra when the daemon is down', () => {
    const add = vi.fn();
    const close = vi.fn();
    daemonStatusUpdate('down', add, close);
    expect(dispatch).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'set-daemon-status', status: 'down' }),
    );
    expect(add).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'daemon-not-connected', type: 'error' }),
    );
  });

  it('closes the not-connected toast once daemon info is available', () => {
    const close = vi.fn();
    daemonStatusUpdate(okStatus(info('mainnet')), vi.fn(), close);
    expect(close).toHaveBeenCalledWith('daemon-not-connected');
  });
});
