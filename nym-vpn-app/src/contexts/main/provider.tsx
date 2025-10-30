import { invoke } from '@tauri-apps/api/core';
import React, { useEffect, useReducer } from 'react';
import { InitState, SystemMessage } from '../../types';
import { initFirstBatch, initSecondBatch } from '../../state/init';
import { useTauriEvents } from '../../state/useTauriEvents';
import { useInAppNotify } from '../in-app-notification';
import { daemonStatusUpdate, networkEnvChanged } from '../../state/helper';
import { CCache } from '../../cache';
import { MainDispatchContext, MainStateContext } from './context';
import { initialState, reducer } from './reducer';

let initialized = false;
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
  });

  const { push } = useInAppNotify();
  useTauriEvents(dispatch, push);

  // initialize app state
  useEffect(() => {
    if (initialized) {
      return;
    }
    initialized = true;
    daemonStatusUpdate(init.vpnd, dispatch, push);
    networkEnvChanged(init.vpnd).then(async (changed) => {
      if (changed) {
        console.info('network env changed, clearing cache');
        await CCache.clear();
      }
    });

    // this first batch is needed to ensure the app is fully initialized and ready
    initFirstBatch(dispatch, init).then(() => {
      console.log('init of 1st batch done');
      dispatch({ type: 'init-done' });
    });

    // this second batch is not needed for the app to be fully
    // functional, and continue loading in the background
    initSecondBatch(dispatch, init).then(() => {
      console.log('init of 2nd batch done');
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (
      systemMessageInit ||
      init.vpnd === 'down' ||
      state.daemonStatus === 'down'
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
