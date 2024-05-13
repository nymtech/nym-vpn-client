import { useCallback, useEffect } from 'react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api';
import { useNavigate } from 'react-router-dom';
import { motion } from 'framer-motion';
import { useMainDispatch, useMainState } from '../../contexts';
import { CmdError, StateDispatch } from '../../types';
import { routes } from '../../router';
import { Button } from '../../ui';
import NetworkModeSelect from './NetworkModeSelect';
import ConnectionStatus from './ConnectionStatus';
import HopSelect from './HopSelect';

function Home() {
  const {
    error,
    state,
    loading,
    entryNodeLocation,
    exitNodeLocation,
    entrySelector,
    daemonStatus,
  } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const navigate = useNavigate();
  const { t } = useTranslation('home');

  const handleClick = async () => {
    dispatch({ type: 'disconnect' });
    if (state === 'Connected') {
      invoke('disconnect')
        .then((result) => {
          console.log('disconnect result');
          console.log(result);
        })
        .catch((e: CmdError) => {
          console.warn('backend error:', e);
          dispatch({ type: 'set-error', error: e.message });
        });
    } else if (state === 'Disconnected') {
      dispatch({ type: 'connect' });
      invoke('connect')
        .then((result) => {
          console.log('connect result');
          console.log(result);
        })
        .catch((e: CmdError) => {
          console.warn('backend error:', e);
          dispatch({ type: 'set-error', error: e.message });
        });
    }
  };

  useEffect(() => {
    if (error?.includes('invalid credential')) {
      navigate(routes.credential);
      dispatch({ type: 'reset-error' });
    }
  }, [error, dispatch, navigate]);

  const getButtonText = useCallback(() => {
    switch (state) {
      case 'Connected':
        return t('disconnect');
      case 'Disconnected':
        return t('connect');
      case 'Connecting':
        return <span className="font-icon text-xl font-medium">autorenew</span>;
      case 'Disconnecting':
        return <span className="font-icon text-xl font-medium">autorenew</span>;
      case 'Unknown':
        return t('status.unknown');
    }
  }, [state, t]);

  const getButtonColor = () => {
    if (state === 'Disconnected' || state === 'Connecting') {
      return 'melon';
    } else if (state === 'Connected' || state === 'Disconnecting') {
      return 'cornflower';
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
        <ConnectionStatus />
      </div>
      <div className="flex flex-col justify-between gap-y-8 select-none">
        <div className="flex flex-col justify-between gap-y-4">
          <NetworkModeSelect />
          <div className="flex flex-col gap-6">
            <div className="mt-3 text-base font-semibold cursor-default">
              {t('select-node-title')}
            </div>
            <div className="flex flex-col gap-5">
              {entrySelector && (
                <HopSelect
                  nodeLocation={entryNodeLocation}
                  onClick={() => navigate(routes.entryNodeLocation)}
                  nodeHop="entry"
                  disabled={state !== 'Disconnected'}
                />
              )}
              <HopSelect
                nodeLocation={exitNodeLocation}
                onClick={() => navigate(routes.exitNodeLocation)}
                nodeHop="exit"
                disabled={state !== 'Disconnected'}
              />
            </div>
          </div>
        </div>
        <Button
          onClick={handleClick}
          color={getButtonColor()}
          disabled={loading || daemonStatus !== 'Ok'}
          className={clsx(
            daemonStatus !== 'Ok' &&
              'opacity-50 disabled:opacity-50 hover:opacity-50',
          )}
        >
          {getButtonText()}
        </Button>
      </div>
    </motion.div>
  );
}

export default Home;
