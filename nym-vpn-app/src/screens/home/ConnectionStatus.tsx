import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { useMainState } from '../../contexts';
import { setToString } from '../../util';
import { useI18nError } from '../../hooks';
import ConnectionBadge from './ConnectionBadge';
import ConnectionTimer from './ConnectionTimer';

function ConnectionStatus() {
  const state = useMainState();
  const [showBadge, setShowBadge] = useState(true);
  const loading =
    state.state === 'Connecting' || state.state === 'Disconnecting';

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

  return (
    <div className="h-full min-h-52 flex flex-col justify-center items-center gap-y-2">
      <div className="flex flex-1 items-end cursor-default select-none">
        {showBadge && <ConnectionBadge state={state.state} />}
      </div>
      <div className="w-full flex flex-col flex-1 items-center overflow-hidden">
        {loading && state.progressMessages.length > 0 && !state.error && (
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.1, ease: 'easeOut' }}
            className="w-4/5 h-2/3 overflow-auto break-words text-center cursor-default select-none"
          >
            <p className="text-sm text-dim-gray dark:text-mercury-mist font-bold">
              {t(
                `connection-progress.${
                  state.progressMessages[state.progressMessages.length - 1]
                }`,
                {
                  ns: 'backendMessages',
                },
              )}
            </p>
          </motion.div>
        )}
        {state.state === 'Connected' && <ConnectionTimer />}
        {state.error && (
          <motion.div
            initial={{ opacity: 0, scale: 0.9, translateX: -8 }}
            animate={{ opacity: 1, scale: 1, translateX: 0 }}
            transition={{ duration: 0.2, ease: 'easeOut' }}
            className="w-4/5 h-2/3 overflow-auto break-words text-center cursor-default"
          >
            <p className="text-sm text-teaberry font-bold">
              {state.error.key ? tE(state.error.key) : state.error.message}
            </p>
            {state.error.data && (
              <p className="text-sm text-teaberry font-bold text-left">
                {setToString(state.error.data)}
              </p>
            )}
          </motion.div>
        )}
      </div>
    </div>
  );
}

export default ConnectionStatus;
