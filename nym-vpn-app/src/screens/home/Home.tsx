import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { type } from '@tauri-apps/plugin-os';
import { useNavigate } from 'react-router';
import clsx from 'clsx';
import { motion } from 'motion/react';
import {
  useMainDispatch,
  useMainState,
  useNodeListState,
} from '../../contexts';
import { BackendError, StateDispatch } from '../../types';
import { routes } from '../../router';
import { Button } from '../../ui';
import { capFirst } from '../../util';
import { kvGet } from '../../kvStore';
import NetworkModeSelect from './NetworkModeSelect';
import TunnelState from './TunnelState';
import HopSelect from './HopSelect';
import NetworkUpdateDialog from './NetworkUpdateDialog';
import UpdateDialog from './UpdateDialog';
import {
  ACTION_TYPE as STREAMING_OPTIMIZED_LABEL_ACTION_TYPE,
  FEATURE_KEY as STREAMING_OPTIMIZED_LABEL_FEATURE_KEY,
  StreamingOptimizedLabel,
} from './new-feature-alert/streaming-optimized-label';
import { setFeatureSeen } from './new-feature-alert/utils';

const updaterEnabled = window._APP.updaterEnabled;
const devMode = window._APP.devMode;
const defaultQuic = window._APP.defaultQuic;
const os = type();
let welcomeInit = false;
let compatChecked = false;

function Home() {
  const {
    state,
    tunnel,
    connectingState,
    accountState,
    entryNode,
    exitNode,
    daemonStatus,
    account,
    networkCompat,
    welcomeChecked,
    backendFlags,
  } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { reset: resetNodeList } = useNodeListState();
  const navigate = useNavigate();
  const { t } = useTranslation('home');
  const loading = state === 'disconnecting';
  const hopSelectDisabled = daemonStatus === 'down' || state !== 'disconnected';
  const needAPlan =
    daemonStatus !== 'down' &&
    state === 'disconnected' &&
    account &&
    (accountState === 'no-subscription' ||
      accountState === 'bandwidth-exceeded');

  const entryGwId = tunnel?.entryGwId || connectingState?.entryGwId || null;
  const exitGwId = tunnel?.exitGwId || connectingState?.exitGwId || null;

  const [isDialogUpdateOpen, setIsDialogUpdateOpen] = useState(false);

  const handleClick = async () => {
    if (state === 'disconnected' && !account) {
      navigate(routes.login);
      return;
    }
    if (needAPlan) {
      navigate(routes.selectPlan);
      return;
    }
    dispatch({ type: 'disconnect' });
    if (
      state === 'connected' ||
      state === 'connecting' ||
      state === 'offline-auto-reconnect' ||
      state === 'error'
    ) {
      console.info('disconnect');
      if (state === 'connecting') {
        dispatch({ type: 'new-progress-message', message: 'canceling' });
      }
      invoke('disconnect')
        .then((result) => {
          console.log(result);
        })
        .catch((e: unknown) => {
          dispatch({ type: 'set-error', error: e as BackendError });
        });
    } else if (state === 'disconnected') {
      console.info('connect');
      dispatch({ type: 'reset-error' });
      dispatch({ type: 'connect' });
      let savedQuic = await kvGet<boolean>('quic-enabled');
      if (savedQuic === undefined) {
        savedQuic = defaultQuic;
      }
      invoke('connect', {
        entry: entryNode,
        exit: exitNode,
        quic: backendFlags.quic && savedQuic,
      })
        .then((result) => {
          console.log(result);
        })
        .catch((e: unknown) => {
          dispatch({ type: 'set-error', error: e as BackendError });
        });
    }
  };

  useEffect(() => {
    if (devMode || compatChecked) {
      return;
    }
    if (
      networkCompat &&
      (networkCompat.core === false || networkCompat.tauri === false)
    ) {
      // if either core or tauri is not compatible, show the update dialog
      compatChecked = true;
      setIsDialogUpdateOpen(true);
    }
  }, [networkCompat]);

  useEffect(() => {
    if (welcomeInit) {
      return;
    }
    welcomeInit = true;
    if (!welcomeChecked) {
      navigate(routes.welcome);
    }
  }, [navigate, welcomeChecked]);

  const getButtonText = useCallback(() => {
    const stop = capFirst(t('stop', { ns: 'glossary' }));
    const cancel = capFirst(t('cancel', { ns: 'glossary' }));
    if (needAPlan) {
      return t('get-started');
    }
    switch (state) {
      case 'connected':
        return t('disconnect');
      case 'disconnected':
      case 'unknown':
        return t('connect');
      case 'connecting':
        return stop;
      case 'disconnecting':
        return null;
      case 'offline':
        return t('connect');
      case 'offline-auto-reconnect':
        return stop;
      case 'error':
        return cancel;
    }
  }, [state, t, needAPlan]);

  const getButtonColor = () => {
    switch (state) {
      case 'disconnected':
      case 'offline':
        return 'malachite';
      case 'connected':
      case 'connecting':
      case 'offline-auto-reconnect':
      case 'disconnecting':
      case 'error':
        return 'red';
      case 'unknown':
        return 'gray';
    }
  };

  const goToNodeList = (hop: 'entry' | 'exit') => {
    if (hop === 'entry') {
      resetNodeList('entry');
      navigate(routes.entryNodeLocation);
    } else {
      resetNodeList('exit');
      navigate(routes.exitNodeLocation);
      setFeatureSeen(
        dispatch,
        STREAMING_OPTIMIZED_LABEL_ACTION_TYPE,
        STREAMING_OPTIMIZED_LABEL_FEATURE_KEY,
      );
    }
  };

  return (
    <>
      {welcomeChecked && updaterEnabled && <UpdateDialog />}
      {os !== 'windows' && (
        <NetworkUpdateDialog
          isOpen={isDialogUpdateOpen}
          onClose={() => setIsDialogUpdateOpen(false)}
          appUpdate={!networkCompat?.tauri}
          daemonUpdate={!networkCompat?.core}
        />
      )}
      <motion.div
        initial={{ opacity: 0, x: '-1rem' }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.2, ease: 'easeOut' }}
        className="sm:max-w-lg h-full flex flex-col"
        data-testid="home-container"
      >
        <StreamingOptimizedLabel />

        <div className="grow" data-testid="home-tunnel-state-container">
          <TunnelState />
        </div>
        <div
          className="flex flex-col justify-between gap-y-8 select-none"
          data-testid="home-controls-container"
        >
          <div className="flex flex-col justify-between gap-y-4">
            <NetworkModeSelect />
            <div
              className="flex flex-col gap-6"
              data-testid="home-node-select-section"
            >
              <div
                className="mt-3 text-base font-medium cursor-default"
                data-testid="home-node-select-title"
              >
                {t('select-node-title')}
              </div>
              <div
                className="flex flex-col gap-5"
                data-testid="home-hop-selects-container"
              >
                <HopSelect
                  node={entryNode}
                  gatewayId={entryGwId}
                  onClick={() => goToNodeList('entry')}
                  nodeHop="entry"
                  disabled={hopSelectDisabled}
                  locked={daemonStatus === 'down'}
                />
                <HopSelect
                  node={exitNode}
                  gatewayId={exitGwId}
                  onClick={() => goToNodeList('exit')}
                  nodeHop="exit"
                  disabled={hopSelectDisabled}
                  locked={daemonStatus === 'down'}
                />
              </div>
            </div>
          </div>
          <Button
            onClick={handleClick}
            color={getButtonColor()}
            disabled={loading || daemonStatus === 'down' || state === 'offline'}
            spinner={loading}
            className={clsx(['h-14', loading && 'data-disabled:opacity-80'])}
            data-testid="home-connection-button"
            data-state={state}
          >
            {getButtonText()}
          </Button>
        </div>
      </motion.div>
    </>
  );
}

export default Home;
