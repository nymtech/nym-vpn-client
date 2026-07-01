import { beforeEach, describe, expect, it } from 'vitest';
import { create } from 'zustand';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createMainSlice } from '../createMainSlice';
import { createSocks5Slice } from '../createSocks5Slice';
import type { GatewaysByCountry } from '../../../types';
import type { AppStore } from '../../index';
import { createGatewaysSlice } from './createGatewaysSlice';

// Mirror the real store composition so slices that read across boundaries
// (e.g. `lookupGw` reads `vpnMode` from the main slice) behave as in production.
const makeStore = () =>
  create<AppStore>()((...args) => ({
    ...createMainSlice(...args),
    ...createGatewaysSlice(...args),
    ...createSocks5Slice(...args),
  }));

let store: ReturnType<typeof makeStore>;

const country = (code: string, ids: string[]): GatewaysByCountry =>
  ({
    country: { code },
    gateways: ids.map((id) => ({ id })),
  }) as unknown as GatewaysByCountry;

beforeEach(() => {
  store = makeStore();
});

describe('lookupGw', () => {
  beforeEach(() => {
    store.setState({
      wg: [country('US', ['wg-1']), country('DE', ['wg-2'])],
      mxEntry: [country('US', ['e-1'])],
      mxExit: [country('FR', ['x-1'])],
    });
  });

  it('searches the wg list in wg mode', () => {
    store.setState({ vpnMode: 'wg' });
    expect(store.getState().lookupGw('wg-2', 'entry')?.id).toBe('wg-2');
  });

  it('searches mxEntry / mxExit by hop in mixnet mode', () => {
    store.setState({ vpnMode: 'mixnet' });
    expect(store.getState().lookupGw('e-1', 'entry')?.id).toBe('e-1');
    expect(store.getState().lookupGw('x-1', 'exit')?.id).toBe('x-1');
  });

  it('scopes the search to a country code when given', () => {
    store.setState({ vpnMode: 'wg' });
    expect(store.getState().lookupGw('wg-1', 'entry', 'us')?.id).toBe('wg-1');
    // right id, wrong country → not found
    expect(store.getState().lookupGw('wg-1', 'entry', 'DE')).toBeNull();
  });

  it('returns null for an unknown id', () => {
    store.setState({ vpnMode: 'mixnet' });
    expect(store.getState().lookupGw('nope', 'entry')).toBeNull();
  });
});

describe('fetchGateways', () => {
  it('uses cached gateways without calling the daemon', async () => {
    const cached = [country('US', ['c-1'])];
    let getGatewaysCalled = false;
    mockIPC((cmd) => {
      if (cmd === 'db_get') return { value: cached };
      if (cmd === 'get_gateways') getGatewaysCalled = true;
      return undefined;
    });
    store.setState({ daemonStatus: 'ok' });
    await store.getState().fetchGateways('wg');
    expect(getGatewaysCalled).toBe(false);
    expect(store.getState().wg).toEqual(cached);
    expect(store.getState().wgLoading).toBe(false);
  });

  it('fetches from the daemon on a cache miss and caches the result', async () => {
    const fresh = [country('DE', ['f-1'])];
    const setKeys: unknown[] = [];
    mockIPC((cmd, payload) => {
      if (cmd === 'db_get') return null;
      if (cmd === 'get_gateways') return fresh;
      if (cmd === 'db_set') {
        setKeys.push((payload as { key: string }).key);
      }
      return undefined;
    });
    await store.getState().fetchGateways('mx-entry');
    expect(store.getState().mxEntry).toEqual(fresh);
    expect(setKeys).toContain('cache-mx-entry-gateways');
    expect(store.getState().mxEntryError).toBeNull();
  });

  it('records the error for mx-entry when the daemon fetch fails', async () => {
    mockIPC((cmd) => {
      if (cmd === 'db_get') return null;
      if (cmd === 'get_gateways') throw new Error('boom');
      return undefined;
    });
    await store.getState().fetchGateways('mx-entry');
    expect(store.getState().mxEntry).toEqual([]);
  });

  it('does nothing when a fetch for that type is already in flight', async () => {
    let getGatewaysCalled = false;
    mockIPC((cmd) => {
      if (cmd === 'get_gateways') getGatewaysCalled = true;
      return undefined;
    });
    store.setState({ wgLoading: true });
    await store.getState().fetchGateways('wg');
    expect(getGatewaysCalled).toBe(false);
  });
});
