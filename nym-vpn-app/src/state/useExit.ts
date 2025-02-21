import { invoke } from '@tauri-apps/api/core';
import { exit as processExit } from '@tauri-apps/plugin-process';
import { useMainDispatch, useMainState } from '../contexts';
import { kvFlush } from '../kvStore';
import { StateDispatch } from '../types';

// Hook to exit the app
export function useExit() {
  const state = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  const exit = async () => {
    console.info('app exit');
    if (
      state.state === 'Connected' ||
      state.state === 'Error' ||
      state.state === 'Connecting' ||
      state.state === 'OfflineAutoReconnect'
    ) {
      // TODO add a timeout to prevent the app from hanging
      // in bad disconnect scenarios
      dispatch({ type: 'disconnect' });
      // flush the database to save the current state
      await kvFlush();
      // disconnect from the backend and then exit
      invoke('disconnect')
        .then(async (result) => {
          console.log('disconnect result');
          console.log(result);
          await processExit(0);
        })
        .catch(async (e: unknown) => {
          console.warn('backend error:', e);
          await processExit(1);
        });
    } else {
      await processExit(0);
    }
  };

  return { exit };
}
