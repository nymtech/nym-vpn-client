import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useNavigate } from 'react-router';
import clsx from 'clsx';
import { motion } from 'motion/react';
import { useMainDispatch, useMainState } from '../../contexts';
import { BackendError, StateDispatch } from '../../types';
import { routes } from '../../router';
import { S_STATE } from '../../static';
import { Button } from '../../ui';
import { capFirst } from '../../util';
import NetworkModeSelect from './NetworkModeSelect';
import TunnelState from './TunnelState';
import HopSelect from './HopSelect';
import NetworkUpdateDialog from './NetworkUpdateDialog';
import UpdateDialog from './UpdateDialog';

const updaterEnabled = window._APP.updaterEnabled;
const devMode = window._APP.devMode;
let compatChecked = false;

function Home() {
  const {
    state,
    accountState,
    entryNode,
    exitNode,
    daemonStatus,
    account,
    networkCompat,
  } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
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

  const [isDialogUpdateOpen, setIsDialogUpdateOpen] = useState(false);

  const handleClick = () => {
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
      invoke('connect', { entry: entryNode, exit: exitNode })
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
    if (!S_STATE.welcomeScreenSeen) {
      navigate(routes.welcome);
    }
  }, [navigate]);

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

  return (
    <>
      {updaterEnabled && <UpdateDialog />}
      <NetworkUpdateDialog
        isOpen={isDialogUpdateOpen}
        onClose={() => setIsDialogUpdateOpen(false)}
        appUpdate={!networkCompat?.tauri}
        daemonUpdate={!networkCompat?.core}
      />
      <motion.div
        initial={{ opacity: 0, x: '-1rem' }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.2, ease: 'easeOut' }}
        className="sm:max-w-lg h-full flex flex-col"
        data-testid="home-container"
      >
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
                  onClick={() => navigate(routes.entryNodeLocation)}
                  nodeHop="entry"
                  disabled={hopSelectDisabled}
                  locked={daemonStatus === 'down'}
                />
                <HopSelect
                  node={exitNode}
                  onClick={() => navigate(routes.exitNodeLocation)}
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
