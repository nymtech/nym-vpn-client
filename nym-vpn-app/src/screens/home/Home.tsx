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
  const { state, entryNodeLocation, exitNodeLocation, daemonStatus, account } =
    useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const navigate = useNavigate();
  const { t } = useTranslation('home');
  const loading = state === 'Disconnecting';

  const handleClick = () => {
    if (state === 'Disconnected' && !account) {
      navigate(routes.login);
      return;
    }
    dispatch({ type: 'disconnect' });
    if (state === 'Connected' || state === 'Connecting') {
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
      dispatch({ type: 'connect' });
      invoke('connect', { entry: entryNodeLocation, exit: exitNodeLocation })
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
      const seen = await kvGet<boolean>('WelcomeScreenSeen');
      if (!seen) {
        navigate(routes.welcome);
      }
    };
    showWelcomeScreen();
  }, [navigate]);

  const getButtonText = useCallback(() => {
    switch (state) {
      case 'Connected':
        return t('disconnect');
      case 'Disconnected':
        return t('connect');
      case 'Connecting':
        return capFirst(t('stop', { ns: 'glossary' }));
      case 'Disconnecting':
        return null;
      default:
        return '-';
    }
  }, [state, t]);

  const getButtonColor = () => {
    switch (state) {
      case 'Disconnected':
        return 'malachite';
      case 'Connecting':
        return 'gray';
      case 'Connected':
      case 'Disconnecting':
        return 'cornflower';
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
                country={entryNodeLocation}
                onClick={() => navigate(routes.entryNodeLocation)}
                nodeHop="entry"
                disabled={daemonStatus === 'NotOk' || state !== 'Disconnected'}
              />
              <HopSelect
                country={exitNodeLocation}
                onClick={() => navigate(routes.exitNodeLocation)}
                nodeHop="exit"
                disabled={daemonStatus === 'NotOk' || state !== 'Disconnected'}
              />
            </div>
          </div>
        </div>
        <Button
          onClick={handleClick}
          color={getButtonColor()}
          disabled={loading || daemonStatus === 'NotOk'}
          spinner={loading}
          className={clsx(['h-14', loading && 'data-[disabled]:opacity-80'])}
        >
          {getButtonText()}
        </Button>
      </div>
    </motion.div>
  );
}

export default Home;
