import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useMainState } from '../../contexts';
import { AnimateIn } from '../../ui';
import { useI18nError } from '../../hooks';
import ConnectionBadge from './ConnectionBadge';
import ConnectionTimer from './ConnectionTimer';

function ConnectionStatus() {
  const state = useMainState();
  const [showBadge, setShowBadge] = useState(true);

  const { t } = useTranslation('home');
  const { eT } = useI18nError();

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
      <div className="flex flex-1 items-end select-none hover:cursor-default">
        {showBadge && <ConnectionBadge state={state.state} />}
      </div>
      <div className="w-full flex flex-col flex-1 items-center overflow-hidden">
        {state.loading && state.progressMessages.length > 0 && !state.error && (
          <AnimateIn
            from="opacity-0 scale-90"
            to="opacity-100 scale-100"
            duration={100}
            className="w-4/5 h-2/3 overflow-auto break-words text-center"
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
          </AnimateIn>
        )}
        {state.state === 'Connected' && <ConnectionTimer />}
        {state.error && (
          <AnimateIn
            from="opacity-0 scale-90 -translate-x-8"
            to="opacity-100 scale-100 translate-y-0 translate-x-0"
            duration={200}
            className="w-4/5 h-2/3 overflow-auto break-words text-center"
          >
            <p className="text-sm text-teaberry font-bold">
              {state.error.key ? eT(state.error.key) : state.error.message}
            </p>
          </AnimateIn>
        )}
      </div>
    </div>
  );
}

export default ConnectionStatus;
