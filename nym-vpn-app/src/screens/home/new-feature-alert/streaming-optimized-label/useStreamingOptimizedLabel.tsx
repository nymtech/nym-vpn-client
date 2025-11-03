import { useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { ButtonText } from '../../../../ui/index';
import { routes } from '../../../../router';
import {
  useInAppNotify,
  useMainDispatch,
  useMainState,
} from '../../../../contexts';
import { StateDispatch } from '../../../../types/index';
import { setStreamOptimizedLabelSeen } from './utils';

const StreamingOptimizedLabelContent = () => {
  const { t } = useTranslation('notifications');
  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;

  const handleClick = () => {
    setStreamOptimizedLabelSeen(dispatch);
    navigate(routes.exitNodeLocation);
  };

  return (
    <div className="flex flex-row justify-between gap-5 items-center">
      <span className="dark:text-white text-baltic-sea">
        {t('streaming-optimized-label')}
      </span>
      <ButtonText
        color="transparent"
        className="text-malachite"
        onClick={handleClick}
      >
        <span className="">{t('streaming-optimized-label-button')}</span>
      </ButtonText>
    </div>
  );
};

let initialized = false;

function useStreamingOptimizedLabel() {
  const { push } = useInAppNotify();
  const { streamingOptimizedLabelSeen } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  useEffect(() => {
    if (initialized || streamingOptimizedLabelSeen) {
      return;
    }
    initialized = true;
    push({
      close: true,
      clickAway: false,
      duration: 10000, // 10 seconds
      onClose: () => setStreamOptimizedLabelSeen(dispatch),
      content: <StreamingOptimizedLabelContent />,
      id: 'streaming-optimized-label',
    });
  }, [push, dispatch, streamingOptimizedLabelSeen]);
}

export default useStreamingOptimizedLabel;
