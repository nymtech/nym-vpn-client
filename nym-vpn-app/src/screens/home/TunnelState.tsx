import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { useMainState } from '../../contexts';
import { setToString } from '../../util';
import { useI18nAccountState, useI18nError } from '../../hooks';
import { AppError } from '../../types';
import ConnectionBadge from './ConnectionBadge';
import ConnectionTimer from './ConnectionTimer';

function TunnelState() {
  const {
    state,
    error,
    progressMessages,
    tunnelError,
    retryAttempt,
    accountState,
  } = useMainState();
  const [showBadge, setShowBadge] = useState(true);
  const loading = state === 'connecting' || state === 'disconnecting';
  const isAccountError =
    accountState === 'max-device-reached' ||
    accountState === 'no-subscription' ||
    accountState === 'bandwidth-exceeded' ||
    accountState === 'status-not-active';
  const isError = tunnelError || error || isAccountError;
  const isOffline = state === 'offline' || state === 'offline-auto-reconnect';
  const showRetryAttempt = state === 'connecting' && !error && retryAttempt > 0;
  const showProgressMsg =
    loading && progressMessages.length > 0 && !error && !showRetryAttempt;

  const { t } = useTranslation('home');
  const { tE } = useI18nError();
  const { tA } = useI18nAccountState();

  useEffect(() => {
    // Quickly hide and show badge when state changes to trigger
    // the animation of state transitions
    setShowBadge(false);
    const timer = setTimeout(() => {
      setShowBadge(true);
    }, 1);

    return () => clearTimeout(timer);
  }, [state]);

  const generalError = (error: AppError) => (
    <>
      <p data-testid="tunnel-error-key">
        {error.key ? tE(error.key) : error.message}
      </p>
      {error.data && (
        <p className="text-left" data-testid="tunnel-error-data">
          {setToString(error.data)}
        </p>
      )}
    </>
  );

  const InfoMessage = (message: string) => (
    <motion.div
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.1, ease: 'easeOut' }}
      className="w-4/5 h-2/3 overflow-auto break-words text-center cursor-default select-none"
      data-testid="tunnel-info-message"
    >
      <p className="text-base text-iron dark:text-bombay">{message}</p>
    </motion.div>
  );

  const getError = () => {
    // prioritize tunnel error first, then account error and finally any general error
    if (tunnelError) {
      return <p data-testid="tunnel-specific-error">{tE(tunnelError)}</p>;
    }
    if (isAccountError) {
      return <p data-testid="account-specific-error">{tA(accountState)}</p>;
    }
    if (error) {
      return generalError(error);
    }
  };

  return (
    <div
      className="h-full min-h-52 flex flex-col justify-center items-center gap-y-2 cursor-default"
      data-testid="tunnel-state-container"
    >
      <div
        className="flex flex-1 items-end cursor-default select-none"
        data-testid="tunnel-badge-container"
      >
        {showBadge && <ConnectionBadge state={state} />}
      </div>
      <div
        className="w-full flex flex-col flex-1 items-center overflow-hidden"
        data-testid="tunnel-details-container"
      >
        {showProgressMsg &&
          InfoMessage(
            t(
              `connection-progress.${
                progressMessages[progressMessages.length - 1]
              }`,
              {
                ns: 'backendMessages',
              },
            ),
          )}
        {showRetryAttempt &&
          InfoMessage(
            t('connection-attempt', {
              ns: 'backendMessages',
              count: retryAttempt,
            }),
          )}
        {isOffline &&
          !isError &&
          InfoMessage(
            t(
              state === 'offline'
                ? 'offline-message'
                : 'offline-reconnect-message',
              { ns: 'home' },
            ),
          )}
        {state === 'connected' && <ConnectionTimer />}
        {isError && (
          <motion.div
            initial={{ opacity: 0, scale: 0.9, translateX: -8 }}
            animate={{ opacity: 1, scale: 1, translateX: 0 }}
            transition={{ duration: 0.2, ease: 'easeOut' }}
            className="w-4/5 h-2/3 overflow-auto break-words text-center cursor-default text-aphrodisiac"
            data-testid="tunnel-error-container"
          >
            {getError()}
          </motion.div>
        )}
      </div>
    </div>
  );
}

export default TunnelState;
