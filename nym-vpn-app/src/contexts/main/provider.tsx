import { invoke } from '@tauri-apps/api/core';
import React, { useEffect, useReducer } from 'react';
import { sleep } from '../../util';
import { Cli, NetworkEnv, SystemMessage } from '../../types';
import { initFirstBatch, initSecondBatch } from '../../state/init';
import { useTauriEvents } from '../../state/useTauriEvents';
import { S_STATE } from '../../static';
import { CCache } from '../../cache';
import { kvGet, kvSet } from '../../kvStore';
import { useInAppNotify } from '../in-app-notification';
import { MainDispatchContext, MainStateContext } from './context';
import { initialState, reducer } from './reducer';

let initialized = false;

type Props = {
  children?: React.ReactNode;
};

function MainStateProvider({ children }: Props) {
  const [state, dispatch] = useReducer(reducer, initialState);
  const { networkEnv } = state;

  const { push } = useInAppNotify();
  useTauriEvents(dispatch, push);

  // const { t } = useTranslation();

  // initialize app state
  useEffect(() => {
    if (initialized) {
      return;
    }
    initialized = true;

    // this first batch is needed to ensure the app is fully
    // initialized and ready, once done splash screen is removed
    // and the UI is shown
    initFirstBatch(dispatch, push).then(async () => {
      console.log('init of 1st batch done');
      dispatch({ type: 'init-done' });
      const args = await invoke<Cli>(`cli_args`);
      // skip the animation if NOSPLASH is set
      if (import.meta.env.APP_NOSPLASH || args.nosplash) {
        return;
      }
      // wait for the splash screen to be visible for a short time
      // as init phase is very fast
      // duration → 700ms
      await sleep(700);
      const splash = document.getElementById('splash');
      if (splash) {
        // starts the fade out animation
        splash.style.opacity = '0';
        // fade out animation duration is set to 150ms, so we wait 300ms
        // to ensure it's done before removing the splash screen
        await sleep(300);
        splash.remove();
        console.log('splash animation done');
      }
    });

    // this second batch is not needed for the app to be fully
    // functional, and continue loading in the background
    initSecondBatch(dispatch).then(() => {
      console.log('init of 2nd batch done');
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // whenever the network environment changes (e.i. daemon has been reconfigured),
  // clear cache
  useEffect(() => {
    const handleNetEnvUpdate = async () => {
      const env = await kvGet<NetworkEnv>('last-network-env');
      if (env === networkEnv) {
        return;
      }
      console.info(`network env changed [${networkEnv}], clearing cache`);
      await kvSet('last-network-env', networkEnv);
      await CCache.clear();
    };

    handleNetEnvUpdate();
  }, [networkEnv]);

  useEffect(() => {
    if (S_STATE.systemMessageInit) {
      return;
    }
    S_STATE.systemMessageInit = true;
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
      } catch (e) {
        console.warn('failed to query system messages:', e);
      }
    };
    querySystemMessages();
  }, [push]);

  return (
    <MainStateContext.Provider value={state}>
      <MainDispatchContext.Provider value={dispatch}>
        {children}
      </MainDispatchContext.Provider>
    </MainStateContext.Provider>
  );
}

export default MainStateProvider;
