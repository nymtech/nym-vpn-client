import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import { emit } from '@tauri-apps/api/event';
import {
  AppDataFromBackend,
  ConnectionState,
  NodeLocationBackend,
} from '../types';
import { ConnectionEvent, DefaultNodeCountry } from '../constants';
import { Country } from '../types';

export function mockTauriIPC() {
  mockWindows('main');

  mockIPC(async (cmd, args) => {
    console.log(`IPC call mocked "${cmd}"`);
    console.log(args);
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
      return 'Disconnected';
    }

    if (cmd === 'get_countries') {
      return new Promise<Country[]>((resolve) =>
        resolve([
          {
            name: 'United States',
            code: 'US',
          },
          {
            name: 'France',
            code: 'FR',
          },
          {
            name: 'Switzerland',
            code: 'CH',
          },
          {
            name: 'Germany',
            code: 'DE',
          },
        ]),
      );
    }

    if (cmd === 'get_node_location') {
      return new Promise<NodeLocationBackend>((resolve) =>
        // resolve('Fastest')
        resolve({
          Country: {
            name: 'France',
            code: 'FR',
          },
        }),
      );
    }

    if (cmd === 'get_fastest_node_location') {
      return new Promise<Country>((resolve) =>
        resolve({
          name: 'France',
          code: 'FR',
        }),
      );
    }

    if (cmd === 'set_root_font_size') {
      return new Promise<void>((resolve) => resolve());
    }

    if (cmd === 'db_get_batch') {
      return new Promise<AppDataFromBackend>((resolve) =>
        resolve({
          monitoring: false,
          autoconnect: false,
          entry_location_enabled: false,
          ui_theme: 'Dark',
          ui_root_font_size: 12,
          vpn_mode: 'TwoHop',
          // entry_node_location: 'Fastest',
          // exit_node_location: 'Fastest',
          entry_node_location: { Country: DefaultNodeCountry },
          exit_node_location: { Country: DefaultNodeCountry },
        }),
      );
    }
  });
}
