import { invoke } from '@tauri-apps/api/core';
import React, { useCallback, useEffect } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { InitState, SystemMessage } from '../../types';
import { initFirstBatch, initSecondBatch } from '../../state/init';
import { useTauriEvents } from '../../state/useTauriEvents';
import { useAccountSummaryOnAccountState } from '../../state/useAccountSummaryOnAccountState';
import { daemonStatusUpdate, networkEnvChanged } from '../../state/helper';
import { CCache } from '../../cache';
import { useToast } from '../../hooks';
import { dispatch, initMainStore, useAppStore } from '../../store';
import IntroSplash from '../../screens/IntroSplash';

let batchesInitialized = false;
let systemMessageInit = false;
let gatewaysInit = false;

const SOCKS5_POLL_INTERVAL = 5000;

type Props = {
  children?: React.ReactNode;
  init: InitState;
};

function MainStateProvider({ children, init }: Props) {
  // Synchronously seed the store with init values before children render
  initMainStore(init);

  const { daemonStatus, initialized, vpnMode } = useAppStore(
    useShallow((s) => ({
      daemonStatus: s.daemonStatus,
      initialized: s.initialized,
      vpnMode: s.vpnMode,
    })),
  );

  const { add, close } = useToast();
  useTauriEvents(add, close);
  useAccountSummaryOnAccountState();

  const initGateways = useCallback(async () => {
    if (gatewaysInit || daemonStatus === 'down') {
      return;
    }
    gatewaysInit = true;
    const { fetchGateways, fetchRecents } = useAppStore.getState();
    // Warm the recents list so the first visit to the node list has it in hand.
    // Fire-and-forget: a failure here is stored on `recentsError` for the
    // recents view alone and must never hold up init or surface a toast.
    fetchRecents(vpnMode);
    if (vpnMode === 'wg') {
      await fetchGateways('wg');
      console.info('[wg] gateways initialized');
    } else {
      await Promise.all([fetchGateways('mx-entry'), fetchGateways('mx-exit')]);
      console.info('[mx-entry + mx-exit] gateways initialized');
    }
  }, [daemonStatus, vpnMode]);

  // initialize app state
  useEffect(() => {
    daemonStatusUpdate(init.vpnd, add, close);
    networkEnvChanged(init.vpnd).then(async (changed) => {
      if (changed) {
        console.info('network env changed, clearing cache');
        await CCache.clear();
      }
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (daemonStatus === 'down' || daemonStatus === 'auth-denied') {
      console.log(
        'daemonStatus is down or auth-denied, skipping initialization',
      );
      batchesInitialized = false;
      gatewaysInit = false;
      return;
    }
    if (batchesInitialized) {
      console.log('batches already initialized, skipping initialization');
      return;
    }
    batchesInitialized = true;

    // this first batch is needed to ensure the app is fully initialized and ready
    Promise.all([initFirstBatch(), initGateways()]).then(() => {
      console.log('init of 1st batch done');
      dispatch({ type: 'init-done' });
    });

    // this second batch is not needed for the app to be fully
    // functional, and continues loading in the background
    initSecondBatch().then(() => {
      console.log('init of 2nd batch done');
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [daemonStatus]);

  useEffect(() => {
    if (daemonStatus === 'down' || daemonStatus === 'auth-denied') {
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
          add({
            title: messages
              .map(({ name, message }) => `${name}: ${message}`)
              .join('\n'),
            type: 'warn',
          });
        }
      } catch {}
    };
    querySystemMessages();
  }, [init.vpnd, daemonStatus, add]);

  useEffect(() => {
    initGateways();
  }, [initGateways]);

  // Socks5 status polling
  useEffect(() => {
    const { refresh } = useAppStore.getState();
    refresh();
    const interval = setInterval(
      () => useAppStore.getState().refresh(),
      SOCKS5_POLL_INTERVAL,
    );
    return () => {
      clearInterval(interval);
    };
  }, []);

  if (
    !initialized &&
    daemonStatus !== 'auth-denied' &&
    daemonStatus !== 'down'
  ) {
    return <IntroSplash theme={init.uiTheme} />;
  }

  return <>{children}</>;
}

export default MainStateProvider;
