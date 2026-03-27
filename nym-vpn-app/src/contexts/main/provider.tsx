import { invoke } from '@tauri-apps/api/core';
import React, { useEffect, useReducer } from 'react';
import { InitState, SystemMessage } from '../../types';
import { initFirstBatch, initSecondBatch } from '../../state/init';
import { useTauriEvents } from '../../state/useTauriEvents';
import { useAccountSummaryOnAccountState } from '../../state/useAccountSummaryOnAccountState';
import { useInAppNotify } from '../in-app-notification';
import { daemonStatusUpdate, networkEnvChanged } from '../../state/helper';
import { CCache } from '../../cache';
import { MainDispatchContext, MainStateContext } from './context';
import { initialState, reducer } from './reducer';

let batchesInitialized = false;
let systemMessageInit = false;

type Props = {
  children?: React.ReactNode;
  init: InitState;
};

function MainStateProvider({ children, init }: Props) {
  const [state, dispatch] = useReducer(reducer, {
    ...initialState,
    vpnMode: init.vpnMode,
    uiTheme: init.uiTheme,
    welcomeChecked: init.welcomeChecked,
    entryNode: init.entryNode,
    exitNode: init.exitNode,
    quic: init.quic,
    enableAdBlocking: init.enableAdBlocking,
    ipv6Support: !init.noIpv6,
    allowLan: init.allowLan,
    customDnsEnabled: init.customDnsEnabled,
    customDns: init.customDns,
    enableLewesProtocol: init.enableLewesProtocol,
    mixnetTrafficConfig: init.mixnetTrafficConfig,
    mixnetTrafficDefaults: init.mixnetTrafficDefaults,
  });

  const { push } = useInAppNotify();
  useTauriEvents(dispatch, push);
  useAccountSummaryOnAccountState(state.accountState, state.initialized, dispatch);

  // initialize app state
  useEffect(() => {
    daemonStatusUpdate(init.vpnd, dispatch, push);
    networkEnvChanged(init.vpnd).then(async (changed) => {
      if (changed) {
        console.info('network env changed, clearing cache');
        await CCache.clear();
      }
    });

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (state.daemonStatus === 'down' || state.daemonStatus === 'auth-denied') {
      console.log(
        'daemonStatus is down or auth-denied, skipping initialization',
      );
      batchesInitialized = false;
      return;
    }
    if (batchesInitialized) {
      console.log('batches already initialized, skipping initialization');
      return;
    }
    batchesInitialized = true;

    // this first batch is needed to ensure the app is fully initialized and ready
    initFirstBatch(dispatch).then(() => {
      console.log('init of 1st batch done');
      dispatch({ type: 'init-done' });
    });

    // this second batch is not needed for the app to be fully
    // functional, and continue loading in the background
    initSecondBatch(dispatch).then(() => {
      console.log('init of 2nd batch done');
    });

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [state.daemonStatus]);

  useEffect(() => {
    if (state.daemonStatus === 'down' || state.daemonStatus === 'auth-denied') {
      systemMessageInit = false;
      return;
    }

    if (
      systemMessageInit ||
      init.vpnd === 'down' ||
      init.vpnd === 'authDenied'
    ) {
      return;
    }
    systemMessageInit = true;
    const querySystemMessages = async () => {
      try {
        const messages = await invoke<SystemMessage[]>('system_messages');
        if (messages.length > 0) {
          console.info('system messages', messages);
          push({
            message: messages
              .map(({ name, message }) => `${name}: ${message}`)
              .join('\n'),
            close: true,
            duration: 10000,
            type: 'warn',
          });
        }
      } catch {}
    };
    querySystemMessages();
  }, [init.vpnd, push, state.daemonStatus]);

  return (
    <MainStateContext.Provider value={state}>
      <MainDispatchContext.Provider value={dispatch}>
        {children}
      </MainDispatchContext.Provider>
    </MainStateContext.Provider>
  );
}

export default MainStateProvider;
