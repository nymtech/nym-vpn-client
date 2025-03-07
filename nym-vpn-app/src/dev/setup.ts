import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import { InvokeArgs } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import {
  AccountLinks,
  Cli,
  DbKey,
  GatewayType,
  GatewaysByCountry,
  NetworkCompat,
  Tunnel,
  TunnelData,
  VpndStatus,
} from '../types';
import { TunnelStateEvent } from '../constants';

// some data
import wgGwJson from './wg-gw.json';
import mxEntryGwJson from './mx-entry-gw.json';
import mxExitGwJson from './mx-exit-gw.json';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type MockIpcFn = (cmd: string, payload?: InvokeArgs) => Promise<any>;

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
      return new Promise<VpndStatus>((resolve) =>
        resolve({
          ok: {
            version: '0.0.0',
            network: 'mainnet',
          },
        }),
      );
    }

    if (cmd === 'startup_error') {
      return null;
    }

    if (cmd === 'connect') {
      await emit(TunnelStateEvent, { state: { connecting: null } });
      return new Promise<null>((resolve) =>
        setTimeout(async () => {
          const tunnel: Tunnel = {
            entryGwId: '1234',
            exitGwId: '5678',
            connectedAt: Date.now(),
            data: {} as unknown as TunnelData,
          };
          await emit(TunnelStateEvent, { state: { connected: tunnel } });
          resolve(null);
        }, 1),
      );
    }
    if (cmd === 'disconnect') {
      await emit(TunnelStateEvent, { state: { disconnecting: null } });
      return new Promise<null>((resolve) =>
        setTimeout(async () => {
          await emit(TunnelStateEvent, { state: 'disconnected' });
          resolve(null);
        }, 1),
      );
    }
    if (cmd === 'get_tunnel_state') {
      return { state: 'disconnected' };
    }

    if (cmd === 'get_gateways') {
      return new Promise<GatewaysByCountry[]>((resolve) => {
        switch ((args as Record<string, unknown>).nodeType as GatewayType) {
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
      switch ((args as Record<string, unknown>).key as DbKey) {
        case 'ui-root-font-size':
          res = 12;
          break;
        case 'ui-theme':
          res = 'Dark';
          break;
        case 'welcome-screen-seen':
          res = true;
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
        }),
      );
    }

    if (cmd === 'is_account_stored') {
      return new Promise<boolean>((resolve) => resolve(false));
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

    // if (cmd === 'add_account') {
    //   return new Promise<boolean>((_, reject) => reject(new Error('nope')));
    // }

    if (cmd === 'forget_account') {
      // return new Promise<void>((_, reject) => reject(new Error('oupsy')));
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
        }),
      );
    }

    if (cmd === 'network_compat') {
      return new Promise<NetworkCompat>((resolve) =>
        resolve({
          tauri: true,
          core: true,
        }),
      );
    }

    if (cmd === 'env') {
      return new Promise((resolve) =>
        resolve({
          DEV_MODE: true,
        }),
      );
    }
  }) as MockIpcFn);
}
