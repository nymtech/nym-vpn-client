import { invoke } from '@tauri-apps/api/core';
import { StateCreator } from 'zustand';
import type {
  HttpRpcSettings,
  SelectedNode,
  Socks5Settings,
  Socks5Status,
} from '../../types';
import type { BoundStore } from '../types';

export type Socks5Slice = {
  status: Socks5Status | null;
  isLoading: boolean;
  refresh: () => Promise<void>;
  enable: (
    socks5Settings: Socks5Settings,
    httpRpcSettings: HttpRpcSettings,
    exit: SelectedNode,
  ) => Promise<void>;
  disable: () => Promise<void>;
};

export const createSocks5Slice: StateCreator<
  BoundStore,
  [],
  [],
  Socks5Slice
> = (set, get) => ({
  status: null,
  isLoading: false,

  refresh: async () => {
    try {
      const result = await invoke<Socks5Status>('get_socks5_status');
      set({ status: result });
    } catch {
      // silently ignore - status polling may fail intermittently
    }
  },

  enable: async (socks5Settings, httpRpcSettings, exit) => {
    if (get().isLoading) {
      console.warn(
        'SOCKS5 enable already in progress, ignoring duplicate call',
      );
      return;
    }
    set({ isLoading: true });
    try {
      await invoke('enable_socks5', { socks5Settings, httpRpcSettings, exit });
      await get().refresh();
    } catch (error) {
      console.error('Failed to enable SOCKS5 proxy:', error);
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  disable: async () => {
    if (get().isLoading) {
      console.warn(
        'SOCKS5 disable already in progress, ignoring duplicate call',
      );
      return;
    }
    set({ isLoading: true });
    try {
      await invoke('disable_socks5');
      await get().refresh();
    } catch (error) {
      console.error('Failed to disable SOCKS5 proxy:', error);
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },
});
