import { useCallback, useEffect, useMemo, useReducer } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  BackendError,
  GatewayType,
  GatewaysByCountry,
  StateDispatch,
} from '../../types';
import { useMainDispatch, useMainState } from '../main';
import { CCache } from '../../cache';
import { DefaultCountry, GatewaysCacheDuration } from '../../constants';
import { kvSet } from '../../kvStore';
import { S_STATE } from '../../static';
import { exists, getStateProps, gwTypeToCacheKey } from './util';
import { GatewaysContext, initialState } from './context';
import { reducer } from './reducer';
import { GatewaysState } from './types';

let initialized = false;

type GatewaysStateProviderProps = {
  children: React.ReactNode;
};

function GatewaysProvider({ children }: GatewaysStateProviderProps) {
  const [state, dispatch] = useReducer(reducer, initialState);

  const { entryNode, exitNode, daemonStatus } = useMainState();
  const mainDispatch = useMainDispatch() as StateDispatch;

  const checkSelectedNode = useCallback(
    async (gateways: GatewaysByCountry[], nodeType: 'entry' | 'exit') => {
      const node = nodeType === 'entry' ? entryNode : exitNode;
      if (!exists(node, gateways)) {
        console.info(`[${nodeType}] node not available, swap to default`);
        mainDispatch({
          type: 'set-node',
          payload: {
            hop: nodeType,
            node: DefaultCountry,
          },
        });
        await kvSet(`${nodeType}-node`, DefaultCountry);
        // TODO notify user
      }
    },
    [mainDispatch, entryNode, exitNode],
  );

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
          console.warn(`Failed to fetch ${nodeType} gateways:`, e);
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
        console.warn(`no gateways found for ${nodeType}`);
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
      if (gateways.length > 0) {
        if (nodeType === 'mx-entry') {
          await checkSelectedNode(gateways, 'entry');
        } else if (nodeType === 'mx-exit') {
          await checkSelectedNode(gateways, 'exit');
        } else {
          // for wg check both entry and exit as they share the same gateways
          await checkSelectedNode(gateways, 'entry');
          await checkSelectedNode(gateways, 'exit');
        }
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [
      checkSelectedNode,
      daemonStatus,
      state.mxEntryLoading,
      state.mxExitLoading,
      state.wgLoading,
    ],
  );

  // init gateways on app start
  useEffect(() => {
    if (initialized || daemonStatus === 'down') {
      return;
    }
    initialized = true;
    if (S_STATE.vpnModeAtStart === 'wg') {
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
  }, [fetchGateways, daemonStatus]);

  const ctx = useMemo<GatewaysState>(
    () => ({
      ...state,
      fetch: fetchGateways,
    }),
    [state, fetchGateways],
  );

  return (
    <GatewaysContext.Provider value={ctx}>{children}</GatewaysContext.Provider>
  );
}

export default GatewaysProvider;
