import { useCallback, useEffect, useMemo, useReducer } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { BackendError, GatewayType, GatewaysByCountry } from '../../types';
import { useMainState } from '../main';
import { CCache } from '../../cache';
import { GatewaysCacheDuration } from '../../constants';
import { getStateProps, gwTypeToCacheKey } from './util';
import { GatewaysContext, initialState } from './context';
import { reducer } from './reducer';
import { GatewaysState } from './types';

let init = false;

type GatewaysStateProviderProps = {
  children: React.ReactNode;
};

function GatewaysProvider({ children }: GatewaysStateProviderProps) {
  const [state, dispatch] = useReducer(reducer, initialState);

  const { initialized, daemonStatus, vpnMode } = useMainState();

  // use cached values if any, otherwise query from daemon
  const fetchGateways = useCallback(
    async (nodeType: GatewayType) => {
      const { loading } = getStateProps(nodeType);
      if (state[loading]) {
        return;
      }
      dispatch({
        type: 'set-gateways-loading',
        payload: {
          type: nodeType,
          loading: true,
        },
      });
      const cacheKey = gwTypeToCacheKey(nodeType);
      // first try to load from cache
      let gateways = await CCache.get<GatewaysByCountry[]>(cacheKey);

      // fallback to daemon query
      if (!gateways || daemonStatus === 'down') {
        console.info(`fetching gateways for ${nodeType}`);
        try {
          gateways = await invoke<GatewaysByCountry[]>('get_gateways', {
            nodeType,
          });
          await CCache.set(cacheKey, gateways, GatewaysCacheDuration);
        } catch (e) {
          if (nodeType === 'mx-entry') {
            // this also reset loading state
            dispatch({
              type: 'set-gateways-error',
              payload: {
                type: nodeType,
                error: e as BackendError,
              },
            });
          }
        }
      }
      if (!gateways) {
        console.info(`no gateways found for ${nodeType}`);
        gateways = [];
      }
      dispatch({
        type: 'set-gateways',
        payload: {
          type: nodeType,
          gateways,
        },
      });
      dispatch({
        type: 'reset-loading-and-error',
        payload: {
          type: nodeType,
        },
      });
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [daemonStatus, state.mxEntryLoading, state.mxExitLoading, state.wgLoading],
  );

  const findGateway = (
    id: string,
    gateways: GatewaysByCountry[],
    countryCode?: string,
  ) => {
    if (countryCode) {
      const byCountry = gateways.find(
        (c) => c.country.code.toLowerCase() === countryCode.toLowerCase(),
      );
      if (byCountry) {
        return byCountry.gateways.find((gw) => gw.id === id) || null;
      }
      return null;
    }
    for (const byCountry of gateways) {
      const gw = byCountry.gateways.find((g) => g.id === id);
      if (gw) {
        return gw;
      }
    }
    return null;
  };

  const lookupGw = useCallback(
    (id: string, type: 'entry' | 'exit', countryCode?: string) => {
      if (vpnMode === 'wg') {
        return findGateway(id, state.wg, countryCode);
      } else if (type === 'entry') {
        return findGateway(id, state.mxEntry, countryCode);
      } else {
        return findGateway(id, state.mxExit, countryCode);
      }
    },
    [state.mxEntry, state.mxExit, state.wg, vpnMode],
  );

  // init gateways on app start
  useEffect(() => {
    if (!initialized || init || daemonStatus === 'down') {
      return;
    }
    init = true;
    if (vpnMode === 'wg') {
      fetchGateways('wg').then(() => {
        console.info('[wg] gateways initialized');
      });
    } else {
      fetchGateways('mx-entry').then(() => {
        console.info('[mx-entry] gateways initialized');
      });
      fetchGateways('mx-exit').then(() => {
        console.info('[mx-exit] gateways initialized');
      });
    }
  }, [initialized, fetchGateways, daemonStatus, vpnMode]);

  const ctx = useMemo<GatewaysState>(
    () => ({
      ...state,
      fetch: fetchGateways,
      lookupGw,
    }),
    [state, fetchGateways, lookupGw],
  );

  return (
    <GatewaysContext.Provider value={ctx}>{children}</GatewaysContext.Provider>
  );
}

export default GatewaysProvider;
