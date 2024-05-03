import { invoke } from '@tauri-apps/api';
import { exit as processExit } from '@tauri-apps/api/process';
import { useMainDispatch, useMainState } from '../contexts';
import { kvFlush } from '../kvStore';
import { CmdError, StateDispatch } from '../types';

// Hook to exit the app
export function useExit() {
  const state = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  const exit = async () => {
    if (state.state === 'Connected') {
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
        .catch(async (e: CmdError) => {
          console.warn('backend error:', e);
          await processExit(1);
        });
    } else {
      await processExit(0);
    }
  };

  return { exit };
}
