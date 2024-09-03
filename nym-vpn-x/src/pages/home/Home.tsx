import { useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api';
import { useNavigate } from 'react-router-dom';
import clsx from 'clsx';
import { motion } from 'framer-motion';
import { useMainDispatch, useMainState } from '../../contexts';
import { BackendError, StateDispatch } from '../../types';
import { routes } from '../../router';
import { kvGet } from '../../kvStore';
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

  const handleClick = () => {
    dispatch({ type: 'disconnect' });
    if (state === 'Connected') {
      invoke('disconnect')
        .then((result) => {
          console.log('disconnect result');
          console.log(result);
        })
        .catch((e: unknown) => {
          console.warn('backend error:', e);
          dispatch({ type: 'set-error', error: e as BackendError });
        });
    } else if (state === 'Disconnected') {
      dispatch({ type: 'connect' });
      invoke('connect', { entry: entryNodeLocation, exit: exitNodeLocation })
        .then((result) => {
          console.log('connect result');
          console.log(result);
        })
        .catch((e: unknown) => {
          console.warn('backend error:', e);
          dispatch({ type: 'set-error', error: e as BackendError });
        });
    }
  };

  useEffect(() => {
    const handleNoValidCredError = async () => {
      if (error?.key === 'CStateNoValidCredential') {
        const expiry = await kvGet<string>('CredentialExpiry');
        const connectionAttempts = await invoke<number>(
          'get_connection_attempts',
        );
        if (expiry && connectionAttempts == 0) {
          dispatch({ type: 'reset-error' });
        } else {
          navigate(routes.credential);
          dispatch({ type: 'reset-error' });
        }
      }
    };

    handleNoValidCredError().catch((e: unknown) => {
      console.warn('error handling no valid cred error:', e);
    });
  }, [error, dispatch, navigate]);

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
        return null;
      case 'Disconnecting':
        return null;
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
                  disabled={
                    daemonStatus === 'NotOk' || state !== 'Disconnected'
                  }
                />
              )}
              <HopSelect
                nodeLocation={exitNodeLocation}
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
          disabled={loading || daemonStatus !== 'Ok'}
          loading={loading}
          className={clsx(['h-14', loading && 'data-[disabled]:opacity-80'])}
        >
          {getButtonText()}
        </Button>
      </div>
    </motion.div>
  );
}

export default Home;
