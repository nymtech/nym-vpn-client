import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { useMainState } from '../../contexts';
import { setToString } from '../../util';
import { useI18nError } from '../../hooks';
import { AppError } from '../../types';
import ConnectionBadge from './ConnectionBadge';
import ConnectionTimer from './ConnectionTimer';

function TunnelState() {
  const state = useMainState();
  const [showBadge, setShowBadge] = useState(true);
  const loading =
    state.state === 'Connecting' || state.state === 'Disconnecting';
  const isError = state.tunnelError || state.error;
  const isOffline =
    state.state === 'Offline' || state.state === 'OfflineAutoReconnect';

  const { t } = useTranslation('home');
  const { tE } = useI18nError();

  useEffect(() => {
    // Quickly hide and show badge when state changes to trigger
    // the animation of state transitions
    setShowBadge(false);
    const timer = setTimeout(() => {
      setShowBadge(true);
    }, 1);

    return () => clearTimeout(timer);
  }, [state.state]);

  const generalError = (error: AppError) => (
    <>
      <p>{error.key ? tE(error.key) : error.message}</p>
      {error.data && <p className="text-left">{setToString(error.data)}</p>}
    </>
  );

  const InfoMessage = (message: string) => (
    <motion.div
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.1, ease: 'easeOut' }}
      className="w-4/5 h-2/3 overflow-auto break-words text-center cursor-default select-none"
    >
      <p className="text-base text-iron dark:text-bombay">{message}</p>
    </motion.div>
  );

  return (
    <div className="h-full min-h-52 flex flex-col justify-center items-center gap-y-2 cursor-default">
      <div className="flex flex-1 items-end cursor-default select-none">
        {showBadge && <ConnectionBadge state={state.state} />}
      </div>
      <div className="w-full flex flex-col flex-1 items-center overflow-hidden">
        {loading &&
          state.progressMessages.length > 0 &&
          !state.error &&
          InfoMessage(
            t(
              `connection-progress.${
                state.progressMessages[state.progressMessages.length - 1]
              }`,
              {
                ns: 'backendMessages',
              },
            ),
          )}
        {isOffline &&
          !isError &&
          InfoMessage(
            t(
              state.state === 'Offline'
                ? 'offline-message'
                : 'offline-reconnect-message',
              { ns: 'home' },
            ),
          )}
        {state.state === 'Connected' && <ConnectionTimer />}
        {isError && (
          <motion.div
            initial={{ opacity: 0, scale: 0.9, translateX: -8 }}
            animate={{ opacity: 1, scale: 1, translateX: 0 }}
            transition={{ duration: 0.2, ease: 'easeOut' }}
            className="w-4/5 h-2/3 overflow-auto break-words text-center cursor-default text-aphrodisiac"
          >
            {state.tunnelError ? (
              <p>{tE(state.tunnelError)}</p>
            ) : (
              state.error && generalError(state.error)
            )}
          </motion.div>
        )}
      </div>
    </div>
  );
}

export default TunnelState;
