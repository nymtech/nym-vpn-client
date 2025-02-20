import { useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useNavigate } from 'react-router';
import clsx from 'clsx';
import { motion } from 'motion/react';
import { useMainDispatch, useMainState } from '../../contexts';
import { BackendError, StateDispatch } from '../../types';
import { routes } from '../../router';
import { kvGet } from '../../kvStore';
import { Button } from '../../ui';
import { capFirst } from '../../util';
import NetworkModeSelect from './NetworkModeSelect';
import TunnelState from './TunnelState';
import HopSelect from './HopSelect';

function Home() {
  const { state, entryNode, exitNode, daemonStatus, account } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const navigate = useNavigate();
  const { t } = useTranslation('home');
  const loading = state === 'Disconnecting';
  const hopSelectDisabled =
    daemonStatus === 'NotOk' || state !== 'Disconnected';

  const handleClick = () => {
    if (state === 'Disconnected' && !account) {
      navigate(routes.login);
      return;
    }
    dispatch({ type: 'disconnect' });
    if (
      state === 'Connected' ||
      state === 'Connecting' ||
      state === 'OfflineAutoReconnect' ||
      state === 'Error'
    ) {
      console.info('disconnect');
      if (state === 'Connecting') {
        dispatch({ type: 'new-progress-message', message: 'Canceling' });
      }
      invoke('disconnect')
        .then((result) => {
          console.log(result);
        })
        .catch((e: unknown) => {
          console.warn('backend error:', e);
          dispatch({ type: 'set-error', error: e as BackendError });
        });
    } else if (state === 'Disconnected') {
      console.info('connect');
      dispatch({ type: 'reset-error' });
      dispatch({ type: 'connect' });
      invoke('connect', { entry: entryNode, exit: exitNode })
        .then((result) => {
          console.log(result);
        })
        .catch((e: unknown) => {
          console.warn('backend error:', e);
          dispatch({ type: 'set-error', error: e as BackendError });
        });
    }
  };

  useEffect(() => {
    const showWelcomeScreen = async () => {
      const seen = await kvGet<boolean>('welcome-screen-seen');
      if (!seen) {
        navigate(routes.welcome);
      }
    };
    showWelcomeScreen();
  }, [navigate]);

  const getButtonText = useCallback(() => {
    const stop = capFirst(t('stop', { ns: 'glossary' }));
    const cancel = capFirst(t('cancel', { ns: 'glossary' }));
    switch (state) {
      case 'Connected':
        return t('disconnect');
      case 'Disconnected':
        return t('connect');
      case 'Connecting':
        return stop;
      case 'Disconnecting':
        return null;
      case 'Offline':
        return t('connect');
      case 'OfflineAutoReconnect':
        return stop;
      case 'Error':
        return cancel;
    }
  }, [state, t]);

  const getButtonColor = () => {
    switch (state) {
      case 'Disconnected':
      case 'Offline':
        return 'malachite';
      case 'Connecting':
      case 'OfflineAutoReconnect':
        return 'gray';
      case 'Connected':
      case 'Disconnecting':
        return 'cornflower';
      case 'Error':
        return 'red';
      default:
        return 'gray';
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, x: '-1rem' }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.2, ease: 'easeOut' }}
      className="h-full flex flex-col"
    >
      <div className="grow">
        <TunnelState />
      </div>
      <div className="flex flex-col justify-between gap-y-8 select-none">
        <div className="flex flex-col justify-between gap-y-4">
          <NetworkModeSelect />
          <div className="flex flex-col gap-6">
            <div className="mt-3 text-base font-semibold cursor-default">
              {t('select-node-title')}
            </div>
            <div className="flex flex-col gap-5">
              <HopSelect
                node={entryNode}
                onClick={() => navigate(routes.entryNodeLocation)}
                nodeHop="entry"
                disabled={hopSelectDisabled}
              />
              <HopSelect
                node={exitNode}
                onClick={() => navigate(routes.exitNodeLocation)}
                nodeHop="exit"
                disabled={hopSelectDisabled}
              />
            </div>
          </div>
        </div>
        <Button
          onClick={handleClick}
          color={getButtonColor()}
          disabled={loading || daemonStatus === 'NotOk' || state === 'Offline'}
          spinner={loading}
          className={clsx(['h-14', loading && 'data-disabled:opacity-80'])}
        >
          {getButtonText()}
        </Button>
      </div>
    </motion.div>
  );
}

export default Home;
