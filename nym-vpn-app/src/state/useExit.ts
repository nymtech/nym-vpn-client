import { invoke } from '@tauri-apps/api/core';
import { exit as processExit } from '@tauri-apps/plugin-process';
import { dispatch, useAppStore } from '../store';
import { kvFlush } from '../kvStore';

// Hook to exit the app
export function useExit() {
  const tunnelState = useAppStore((s) => s.state);

  const exit = async () => {
    console.info('app exit');
    if (
      tunnelState === 'connected' ||
      tunnelState === 'error' ||
      tunnelState === 'connecting' ||
      tunnelState === 'offline-auto-reconnect'
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
        .catch(async () => {
          await processExit(1);
        });
    } else {
      await processExit(0);
    }
  };

  return { exit };
}
