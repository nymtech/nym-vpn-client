import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import { InvokeArgs } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import {
  AccountLinks,
  Cli,
  DbKey,
  Tunnel,
  TunnelData,
  VpndStatus,
} from '../types';
import { TunnelStateEvent } from '../constants';
import { Country } from '../types';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type MockIpcFn = (cmd: string, payload?: InvokeArgs) => Promise<any>;

export function mockTauriIPC() {
  mockWindows('main');
  window.__TAURI_OS_PLUGIN_INTERNALS__ = {
    // @ts-expect-error mocking os plugin
    os: {
      type: () => 'linux',
    },
  };

  mockIPC((async (cmd, args) => {
    console.log(`IPC call mocked "${cmd}"`);
    console.log(args);

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

    if (cmd === 'get_countries') {
      return new Promise<Country[]>((resolve) =>
        resolve([
          {
            name: 'France',
            code: 'FR',
          },
          {
            name: 'Germany',
            code: 'DE',
          },
          {
            name: 'Switzerland',
            code: 'CH',
          },
          {
            name: 'United States',
            code: 'US',
          },
          {
            name: 'Unknown country with a very long nammmmmmmmeeeeeeeeeeeeeeee',
            code: 'UN',
          },
        ]),
      );
    }

    if (cmd === 'db_get') {
      let res: unknown = undefined;
      if (!args) {
        return;
      }
      switch ((args as Record<string, unknown>).key as DbKey) {
        case 'UiRootFontSize':
          res = 12;
          break;
        case 'UiTheme':
          res = 'Dark';
          break;
        case 'WelcomeScreenSeen':
          res = true;
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

    if (cmd === 'env') {
      return new Promise((resolve) =>
        resolve({
          DEV_MODE: true,
        }),
      );
    }
  }) as MockIpcFn);
}
