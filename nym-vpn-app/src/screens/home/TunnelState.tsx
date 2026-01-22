import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { useMainState } from '../../contexts';
import { setToString } from '../../util';
import {
  useI18nAccountState,
  useI18nError,
  useI18nProgressMsg,
  useI18nTunnelError,
} from '../../hooks';
import { AppError } from '../../types';
import ConnectionBadge from './ConnectionBadge';
import ConnectionTimer from './ConnectionTimer';

function TunnelState() {
  const {
    state,
    error,
    progressMessages,
    tunnelError,
    connectingState,
    accountState,
    accountError,
  } = useMainState();
  const [showBadge, setShowBadge] = useState(true);
  const loading = state === 'connecting' || state === 'disconnecting';
  const isAccountError =
    accountState === 'max-device-reached' ||
    accountState === 'no-subscription' ||
    accountState === 'bandwidth-exceeded' ||
    accountState === 'status-not-active' ||
    accountState === 'error';
  const isError = tunnelError || error || isAccountError;
  const isOffline = state === 'offline' || state === 'offline-auto-reconnect';
  const retryAttempt = connectingState?.retryAttempt || 0;
  const showRetryAttempt = !error && state === 'connecting' && retryAttempt > 0;
  const showConnectingProgress =
    !error && state === 'connecting' && connectingState?.progress !== undefined;
  const showProgressMsg = loading && !error && progressMessages.length > 0;

  const { t } = useTranslation('home');
  const { tE } = useI18nError();
  const { tTE } = useI18nTunnelError();
  const { t: tA } = useI18nAccountState();
  const { t: tP } = useI18nProgressMsg();

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

  const messages = useMemo(() => {
    const msgs = [];
    if (isOffline) {
      msgs.push(
        t(
          state === 'offline' ? 'offline-message' : 'offline-reconnect-message',
          { ns: 'home' },
        ),
      );
      return msgs;
    }
    if (showProgressMsg) {
      msgs.push(tP(progressMessages[progressMessages.length - 1]));
    }
    if (showRetryAttempt) {
      msgs.push(
        t('connection-attempt', {
          ns: 'backend-messages',
          count: retryAttempt,
        }),
      );
    }
    if (showConnectingProgress) {
      msgs.push(tP(connectingState.progress));
    }
    return msgs;
  }, [
    connectingState?.progress,
    isOffline,
    progressMessages,
    retryAttempt,
    showConnectingProgress,
    showProgressMsg,
    showRetryAttempt,
    state,
    t,
    tP,
  ]);

  const InfoMessage = (message: string[]) => (
    <motion.div
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.1, ease: 'easeOut' }}
      className="w-4/5 wrap-break-word text-center cursor-default select-none"
      data-testid="tunnel-info-message"
    >
      {message.map((msg, idx) => (
        <p
          key={`${msg}-${idx}`}
          className="text-base text-iron dark:text-bombay"
        >
          {msg}
        </p>
      ))}
    </motion.div>
  );

  const getError = () => {
    // prioritize tunnel error first, then account error and finally any general error
    if (tunnelError) {
      return <p data-testid="tunnel-specific-error">{tTE(tunnelError)}</p>;
    }
    if (isAccountError) {
      const error = accountError ? tE(accountError.key) : tA(accountState);
      return <p data-testid="account-specific-error">{error}</p>;
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
        className="w-full flex flex-col flex-1 items-center"
        data-testid="tunnel-details-container"
      >
        {!isError && messages.length > 0 && InfoMessage(messages)}
        {state === 'connected' && <ConnectionTimer />}
        {isError && (
          <motion.div
            initial={{ opacity: 0, scale: 0.9, translateX: -8 }}
            animate={{ opacity: 1, scale: 1, translateX: 0 }}
            transition={{ duration: 0.2, ease: 'easeOut' }}
            className="w-4/5 wrap-break-word text-center cursor-default text-aphrodisiac"
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
