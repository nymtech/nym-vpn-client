import { invoke } from '@tauri-apps/api/core';
import { dequal } from 'dequal';
import { StateCreator } from 'zustand';
import {
  BackendError,
  Gateway,
  GatewaysByCountry,
  RecentGateways,
} from '../../../types';
import { CCache } from '../../../cache';
import { GatewaysCacheDuration } from '../../../constants';
import {
  getStateProps,
  gwTypeToCacheKey,
} from '../../../contexts/gateways/util';
import type { MainSlice } from '../createMainSlice';
import type { Socks5Slice } from '../createSocks5Slice';
import type { GatewaysSlice } from './types';

type BoundStore = MainSlice & GatewaysSlice & Socks5Slice;

function findGateway(
  id: string,
  gateways: GatewaysByCountry[],
  countryCode?: string,
): Gateway | null {
  if (countryCode) {
    const byCountry = gateways.find(
      (c) => c.country.code.toLowerCase() === countryCode.toLowerCase(),
    );
    if (byCountry) return byCountry.gateways.find((gw) => gw.id === id) ?? null;
    return null;
  }
  for (const byCountry of gateways) {
    const gw = byCountry.gateways.find((g) => g.id === id);
    if (gw) return gw;
  }
  return null;
}

export const createGatewaysSlice: StateCreator<
  BoundStore,
  [],
  [],
  GatewaysSlice
> = (set, get) => ({
  mxEntry: [],
  mxExit: [],
  wg: [],
  mxEntryLoading: false,
  mxExitLoading: false,
  wgLoading: false,
  mxEntryError: null,
  mxExitError: null,
  wgError: null,
  recents: {
    mixnet: { entry: [], exit: [] },
    wg: { entry: [], exit: [] },
  },
  recentsLoading: { mixnet: false, wg: false },
  recentsError: { mixnet: null, wg: null },

  fetchRecents: async (vpnMode) => {
    if (get().recentsLoading[vpnMode]) return;

    set((s) => ({ recentsLoading: { ...s.recentsLoading, [vpnMode]: true } }));
    try {
      const recents = await invoke<RecentGateways>('get_recent_gateways', {
        vpnMode,
      });
      set((s) => ({
        // Normalised on the way in: consumers index straight into
        // `recents[mode][hop]`, so a payload without both hops would take the
        // whole node list screen down rather than just emptying recents.
        recents: {
          ...s.recents,
          [vpnMode]: {
            entry: recents?.entry ?? [],
            exit: recents?.exit ?? [],
          },
        },
        recentsError: { ...s.recentsError, [vpnMode]: null },
      }));
    } catch (e) {
      console.error('failed to get recent gateways', e);
      set((s) => ({
        recentsError: { ...s.recentsError, [vpnMode]: e as BackendError },
      }));
    } finally {
      set((s) => ({
        recentsLoading: { ...s.recentsLoading, [vpnMode]: false },
      }));
    }
  },

  fetchGateways: async (nodeType) => {
    const {
      gateways: gwKey,
      loading: loadingKey,
      error: errorKey,
    } = getStateProps(nodeType);
    const state = get();
    if (state[loadingKey]) return;

    set({ [loadingKey]: true } as Partial<BoundStore>);
    const cacheKey = gwTypeToCacheKey(nodeType);
    let gateways = await CCache.get<GatewaysByCountry[]>(cacheKey);

    if (!gateways || state.daemonStatus === 'down') {
      console.info(`fetching gateways for ${nodeType}`);
      try {
        gateways = await invoke<GatewaysByCountry[]>('get_gateways', {
          nodeType,
        });
        await CCache.set(cacheKey, gateways, GatewaysCacheDuration);
      } catch (e) {
        if (nodeType === 'mx-entry') {
          set({
            [errorKey]: e as BackendError,
            [loadingKey]: false,
          } as Partial<BoundStore>);
        }
      }
    }
    if (!gateways) {
      console.info(`no gateways found for ${nodeType}`);
      gateways = [];
    }
    if (!dequal(gateways, get()[gwKey])) {
      set({ [gwKey]: gateways } as Partial<BoundStore>);
    }
    set({ [loadingKey]: false, [errorKey]: null } as Partial<BoundStore>);
  },

  lookupGw: (id, type, countryCode) => {
    const { mxEntry, mxExit, wg, vpnMode } = get();
    if (vpnMode === 'wg') return findGateway(id, wg, countryCode);
    return type === 'entry'
      ? findGateway(id, mxEntry, countryCode)
      : findGateway(id, mxExit, countryCode);
  },
});
