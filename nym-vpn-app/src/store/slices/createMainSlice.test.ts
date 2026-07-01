import { beforeEach, describe, expect, it } from 'vitest';
import { type StateCreator, create } from 'zustand';
import type { Tunnel, VpndConfig } from '../../types';
import {
  type MainSlice,
  type StateAction,
  createMainSlice,
  initialState,
} from './createMainSlice';

const makeStore = () =>
  create<MainSlice>()(createMainSlice as unknown as StateCreator<MainSlice>);

let store: ReturnType<typeof makeStore>;
const dispatch = (action: StateAction) => store.getState()._dispatch(action);

beforeEach(() => {
  store = makeStore();
});

describe('createMainSlice initial state', () => {
  it('seeds the documented initial state', () => {
    expect(store.getState().state).toBe('disconnected');
    expect(store.getState().daemonStatus).toBe('down');
    expect(store.getState().vpnMode).toBe('wg');
    expect(store.getState().initialized).toBe(false);
  });
});

describe('_dispatch simple setters', () => {
  it('init-done marks the store initialized', () => {
    dispatch({ type: 'init-done' });
    expect(store.getState().initialized).toBe(true);
  });

  const setterCases: [StateAction, keyof MainSlice, unknown][] = [
    [{ type: 'set-vpn-mode', mode: 'mixnet' }, 'vpnMode', 'mixnet'],
    [{ type: 'set-version', version: '1.2.3' }, 'version', '1.2.3'],
    [{ type: 'set-auto-connect', autoConnect: true }, 'autoConnect', true],
    [{ type: 'set-monitoring', enabled: true }, 'monitoring', true],
    [{ type: 'set-allow-lan', enabled: true }, 'allowLan', true],
    [{ type: 'set-quic', enabled: true }, 'quic', true],
    [{ type: 'set-fronting-mode', mode: 'always' }, 'frontingMode', 'always'],
    [{ type: 'set-custom-dns', dns: ['1.1.1.1'] }, 'customDns', ['1.1.1.1']],
    [{ type: 'set-root-font-size', size: 18 }, 'rootFontSize', 18],
    [{ type: 'set-ui-theme', theme: 'dark' }, 'uiTheme', 'dark'],
  ];

  it.each(setterCases)('%o updates state', (action, key, expected) => {
    dispatch(action);
    expect(store.getState()[key]).toEqual(expected);
  });
});

describe('_dispatch set-daemon-status', () => {
  it('down clears session state and sets a not-connected error', () => {
    dispatch({ type: 'set-tunnel', tunnel: {} as Tunnel });
    dispatch({ type: 'set-daemon-status', status: 'down' });
    const s = store.getState();
    expect(s.daemonStatus).toBe('down');
    expect(s.state).toBe('unknown');
    expect(s.tunnel).toBeNull();
    expect(s.error).toEqual({
      key: 'not-connected-to-daemon',
      message: 'Not connected to the daemon',
    });
  });

  it('auth-denied sets an auth-denied error', () => {
    dispatch({ type: 'set-daemon-status', status: 'auth-denied' });
    expect(store.getState().error).toEqual({
      key: 'auth-denied',
      message: 'Authentication required',
    });
  });

  it('ok clears any prior error', () => {
    dispatch({ type: 'set-daemon-status', status: 'down' });
    dispatch({ type: 'set-daemon-status', status: 'ok' });
    expect(store.getState().daemonStatus).toBe('ok');
    expect(store.getState().error).toBeNull();
  });
});

describe('_dispatch tunnel lifecycle', () => {
  it('connect / disconnect toggle the transient state', () => {
    dispatch({ type: 'connect' });
    expect(store.getState().state).toBe('connecting');
    dispatch({ type: 'disconnect' });
    expect(store.getState().state).toBe('disconnecting');
  });

  it('set-tunnel-connected stores a connectedAt timestamp', () => {
    dispatch({
      type: 'set-tunnel-connected',
      tunnel: { connectedAt: 1_700_000_000 } as unknown as Tunnel,
    });
    const s = store.getState();
    expect(s.state).toBe('connected');
    expect(s.tunnelConnectedAt?.unix()).toBe(1_700_000_000);
  });

  it('set-tunnel-connected without connectedAt falls back to now', () => {
    dispatch({ type: 'set-tunnel-connected', tunnel: {} as Tunnel });
    expect(store.getState().tunnelConnectedAt).not.toBeNull();
  });

  it('set-tunnel-offline picks the reconnect state', () => {
    dispatch({ type: 'set-tunnel-offline', reconnect: true });
    expect(store.getState().state).toBe('offline-auto-reconnect');
    dispatch({ type: 'set-tunnel-offline', reconnect: false });
    expect(store.getState().state).toBe('offline');
  });
});

describe('_dispatch node selection', () => {
  it('routes entry and exit hops to distinct fields', () => {
    const entry = { country: { code: 'US' } } as never;
    const exit = { country: { code: 'DE' } } as never;
    dispatch({ type: 'set-node', payload: { hop: 'entry', node: entry } });
    dispatch({ type: 'set-node', payload: { hop: 'exit', node: exit } });
    expect(store.getState().entryNode).toBe(entry);
    expect(store.getState().exitNode).toBe(exit);
  });
});

describe('_dispatch progress messages', () => {
  it('appends messages in order', () => {
    dispatch({ type: 'new-progress-message', message: 'a' as never });
    dispatch({ type: 'new-progress-message', message: 'b' as never });
    expect(store.getState().progressMessages).toEqual(['a', 'b']);
  });
});

describe('_dispatch system-theme-changed', () => {
  it('updates the UI theme only when following the system', () => {
    dispatch({ type: 'set-theme-mode', mode: 'system' });
    dispatch({ type: 'system-theme-changed', theme: 'dark' });
    expect(store.getState().uiTheme).toBe('dark');
  });

  it('ignores the system theme when a fixed mode is set', () => {
    dispatch({ type: 'set-theme-mode', mode: 'light' });
    dispatch({ type: 'system-theme-changed', theme: 'dark' });
    expect(store.getState().uiTheme).toBe('light');
  });
});

describe('_dispatch nested split-tunnel / geo-exclusion merges', () => {
  it('toggles split-tunnel without dropping the apps list', () => {
    dispatch({ type: 'set-split-tunnel-apps', apps: [{ id: 'x' }] as never });
    dispatch({ type: 'set-enable-split-tunnel', enabled: true });
    expect(store.getState().splitTunnel).toEqual({
      enabled: true,
      apps: [{ id: 'x' }],
    });
  });

  it('merges geo-exclusion fields independently', () => {
    dispatch({ type: 'set-geo-exclusion-enabled', enabled: true });
    dispatch({ type: 'set-geo-exclusion-listen-port', port: 9999 });
    dispatch({
      type: 'set-geo-exclusion-excluded-countries',
      countries: ['DE', 'FR'],
    });
    expect(store.getState().geoExclusion).toEqual({
      enabled: true,
      listenPort: 9999,
      excludedCountries: ['DE', 'FR'],
    });
  });
});

describe('_dispatch update-tunnel-config', () => {
  it('maps config fields into flat state', () => {
    dispatch({
      type: 'update-tunnel-config',
      config: {
        entryNode: 'e',
        exitNode: 'x',
        vpnMode: 'mixnet',
        bridges: true,
        frontingMode: 'always',
        disableIpv6: true,
        allowLan: true,
        enableCustomDns: true,
        customDns: ['9.9.9.9'],
      } as unknown as VpndConfig,
    });
    const s = store.getState();
    expect(s.vpnMode).toBe('mixnet');
    expect(s.quic).toBe(true);
    expect(s.ipv6Support).toBe(false);
    expect(s.customDnsEnabled).toBe(true);
    expect(s.customDns).toEqual(['9.9.9.9']);
  });
});

describe('_dispatch reset', () => {
  it('restores the initial state', () => {
    dispatch({ type: 'set-vpn-mode', mode: 'mixnet' });
    dispatch({ type: 'connect' });
    dispatch({ type: 'reset' });
    expect(store.getState().vpnMode).toBe(initialState.vpnMode);
    expect(store.getState().state).toBe(initialState.state);
  });
});
