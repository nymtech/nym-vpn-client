import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import { InvokeArgs } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import {
  AccountLinks,
  Cli,
  DbKey,
  FeatureFlags,
  GatewayType,
  GatewaysByCountry,
  NetworkCompat,
  TAccountMode,
  TAccountState,
  TTunnelState,
  UiTheme,
  UpdateMetadata,
  VpndStatus,
} from '../types';
import { AccountStateEvent, TunnelStateEvent } from '../constants';

// mocked data
import wgGwJson from './mocked/wg-gw.json';
import mxEntryGwJson from './mocked/mx-entry-gw.json';
import mxExitGwJson from './mocked/mx-exit-gw.json';
import wgTunnel from './mocked/wg-tunnel.json';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type MockIpcFn = (cmd: string, payload?: InvokeArgs) => Promise<any>;
type ArgsObj<T> = Record<string, T>;

// fake state
const appVersion = '0.0.0';
const uiTheme: UiTheme = 'dark';
const lang = 'en';
const showWelcome = false;
// const daemon: VpndStatus = 'down';
const daemon: VpndStatus = {
  ok: {
    version: '0.0.0',
    network: 'mainnet',
    gitCommit: 'ffffff',
  },
};
let tunnelState: TTunnelState = 'disconnected';
// const tunnelState: TTunnelState = { connected: wgTunnel };
// const tunnelState: TTunnelState = { connecting: null };
// const tunnelState: TTunnelState = { disconnecting: null };
// const tunnelState: TTunnelState = { offline: { reconnect: false } };
// const tunnelState: TTunnelState = { offline: { reconnect: true } };
// const tunnelState: TTunnelState = { error: { key: 'internal', data: 'Oupsy something went wrong' } };
let isLoggedIn = true;
let accountState: TAccountState = isLoggedIn ? 'ready' : 'logged-out';
let accountMode: TAccountMode = 'decentralised';
let autostart = true;
// note: compat check is skipped if DEV_MODE=true
const networkCompat: NetworkCompat = {
  tauri: true,
  core: true,
};
const featureFlags: FeatureFlags = {
  quic: true,
  domainFronting: true,
  zknymCredential: true,
  mixnetTuning: false,
};

export function mockTauriIPC() {
  mockWindows('main');
  // @ts-expect-error mocking os plugin
  window.__TAURI_OS_PLUGIN_INTERNALS__ = {
    os_type: 'linux',
    platform: 'linux',
    family: 'unix',
  };
  mockIPC((async (cmd, args) => {
    console.debug(`IPC call mocked "${cmd}"`);
    console.debug(args);

    if (cmd === 'daemon_status') {
      return new Promise<VpndStatus>((resolve) => resolve(daemon));
    }

    if (cmd === 'connect') {
      tunnelState = {
        connecting: {
          tunnelType: 'wg',
          progress: 'connecting-tunnel',
          tunnel: null,
          retryAttempt: 0,
          entryGwId: null,
          exitGwId: null,
        },
      };
      await emit(TunnelStateEvent, { state: tunnelState, error: null });
      return new Promise<null>((resolve) =>
        setTimeout(async () => {
          tunnelState = { connected: wgTunnel as never };
          await emit(TunnelStateEvent, { state: tunnelState, error: null });
          resolve(null);
        }, 2000),
      );
    }
    if (cmd === 'disconnect') {
      tunnelState = { disconnecting: null };
      await emit(TunnelStateEvent, { state: tunnelState, error: null });
      return new Promise<null>((resolve) =>
        setTimeout(async () => {
          tunnelState = 'disconnected';
          await emit(TunnelStateEvent, { state: tunnelState, error: null });
          resolve(null);
        }, 1),
      );
    }
    if (cmd === 'get_tunnel_state') {
      return new Promise<unknown>((resolve) => resolve(tunnelState));
    }

    if (cmd === 'get_gateways') {
      return new Promise<GatewaysByCountry[]>((resolve) => {
        switch ((args as ArgsObj<GatewayType>).nodeType) {
          case 'mx-entry':
            resolve(mxEntryGwJson as GatewaysByCountry[]);
            return;
          case 'mx-exit':
            resolve(mxExitGwJson as GatewaysByCountry[]);
            return;
          case 'wg':
            resolve(wgGwJson as GatewaysByCountry[]);
            return;
        }
      });
    }

    if (cmd === 'db_get') {
      let res: unknown = undefined;
      if (!args) {
        return;
      }
      switch ((args as ArgsObj<DbKey>).key) {
        case 'ui-root-font-size':
          res = 12;
          break;
        case 'ui-theme':
          res = uiTheme;
          break;
        case 'ui-language':
          res = lang;
          break;
        case 'welcome-screen-seen':
          res = !showWelcome;
          break;

        /* 1740391345259 */
        case 'cache-mx-entry-gateways':
          res = {
            expiry: 2740391345259,
            value: mxEntryGwJson,
          };
          break;
        case 'cache-mx-exit-gateways':
          res = {
            expiry: 2740391345259,
            value: mxExitGwJson,
          };
          break;
        case 'cache-wg-gateways':
          res = {
            expiry: 2740391345259,
            value: wgGwJson,
          };
          break;

        default:
          return null;
      }
      return new Promise<unknown>((resolve) => resolve(res));
    }

    if (cmd === 'cli_args') {
      return new Promise<Cli>((resolve) =>
        resolve({
          nosplash: false,
          buildInfo: false,
          cleanLocalFiles: false,
          devMode: true,
          logFile: false,
          logLevel: 'trace',
        }),
      );
    }

    if (cmd === 'is_account_stored') {
      return new Promise<boolean>((resolve) => resolve(isLoggedIn));
    }

    if (cmd === 'get_account_state') {
      return new Promise<TAccountState>((resolve) => resolve(accountState));
    }

    if (cmd === 'get_account_mode') {
      return new Promise<TAccountMode>((resolve) => resolve(accountMode));
    }

    if (cmd === 'get_account_id') {
      return new Promise<string>((resolve) =>
        resolve('xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'),
      );
    }

    if (cmd === 'get_device_id') {
      return new Promise<string>((resolve) =>
        resolve('xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'),
      );
    }

    if (cmd === 'add_account') {
      const mnemonic = (args as ArgsObj<string> | undefined)?.mnemonic?.trim();
      if (!mnemonic) {
        return new Promise<number | null>((_, reject) =>
          reject({
            key: 'empty',
            message: 'Mnemonic is required',
            data: { reason: 'missing mnemonic' },
          }),
        );
      }
      isLoggedIn = true;
      accountState = 'ready';
      accountMode = 'decentralised';
      await emit(AccountStateEvent, accountState);
      return new Promise<number | null>((resolve) => resolve(null));
    }

    if (cmd === 'forget_account') {
      isLoggedIn = false;
      accountState = 'logged-out';
      await emit(AccountStateEvent, accountState);
      return new Promise<void>((resolve) => resolve());
    }

    if (cmd === 'system_messages') {
      return new Promise<object[]>((resolve) => resolve([]));
    }

    if (cmd === 'account_links') {
      return new Promise<AccountLinks>((resolve) =>
        resolve({
          signUp: 'https://xyz.xyz/signup',
          signIn: 'https://xyz.xyz/signin',
          account: 'https://xyz.xyz/account/1234abcd',
        }),
      );
    }

    if (cmd === 'fetch_update') {
      return new Promise<UpdateMetadata>((resolve) =>
        resolve({
          version: '100.0.0',
          currentVersion: appVersion,
        }),
      );
    }

    if (cmd === 'network_compat') {
      return new Promise<NetworkCompat>((resolve) => resolve(networkCompat));
    }

    if (cmd === 'feature_flags') {
      return new Promise((resolve) => resolve(featureFlags));
    }

    if (cmd === 'plugin:autostart|is_enabled') {
      return new Promise((resolve) => resolve(autostart));
    }
    if (cmd === 'plugin:autostart|disable') {
      autostart = false;
    }
    if (cmd === 'plugin:autostart|enable') {
      autostart = true;
    }
    if (cmd === 'plugin:app|version') {
      return new Promise((resolve) => resolve(appVersion));
    }
    if (cmd === 'plugin:clipboard-manager|write_text') {
      console.log(`copied to clipboard: ${(args as ArgsObj<string>).text}`);
    }
  }) as MockIpcFn, { shouldMockEvents: true });
}
