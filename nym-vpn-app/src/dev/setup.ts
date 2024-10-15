import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import { InvokeArgs } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { Cli, ConnectionState, DbKey } from '../types';
import { ConnectionEvent } from '../constants';
import { Country } from '../types';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type MockIpcFn = (cmd: string, payload?: InvokeArgs) => Promise<any>;

export function mockTauriIPC() {
  mockWindows('main');

  mockIPC((async (cmd, args) => {
    console.log(`IPC call mocked "${cmd}"`);
    console.log(args);

    if (cmd === 'startup_error') {
      return null;
    }

    if (cmd === 'connect') {
      await emit(ConnectionEvent, { state: 'Connecting' });
      return new Promise<ConnectionState>((resolve) =>
        setTimeout(async () => {
          await emit(ConnectionEvent, { state: 'Connected' });
          resolve('Connected');
        }, 1),
      );
    }
    if (cmd === 'disconnect') {
      await emit(ConnectionEvent, { state: 'Disconnecting' });
      return new Promise<ConnectionState>((resolve) =>
        setTimeout(async () => {
          await emit(ConnectionEvent, { state: 'Disconnected' });
          resolve('Disconnected');
        }, 1),
      );
    }
    if (cmd === 'get_connection_state') {
      return { state: 'Disconnected' };
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
        case 'UiShowEntrySelect':
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
  }) as MockIpcFn);
}
