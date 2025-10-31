import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { ButtonText, Toast } from '../../../../ui/index';
import { routes } from '../../../../router';
import { useMainDispatch, useMainState } from '../../../../contexts/index';
import { StateDispatch } from '../../../../types/index';
import { setStreamOptimizedLabelSeen } from './utils';

export function StreamingOptimizedLabel() {
  const navigate = useNavigate();
  const { streamingOptimizedLabelSeen } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('notifications');

  const handleClick = () => {
    setStreamOptimizedLabelSeen(dispatch);
    navigate(routes.exitNodeLocation);
  };

  const onOpenChange = (open: boolean) => {
    if (!open) {
      setStreamOptimizedLabelSeen(dispatch);
    }
  };

  if (streamingOptimizedLabelSeen) {
    return null;
  }

  return (
    <Toast
      open={!streamingOptimizedLabelSeen}
      close={true}
      duration={Infinity}
      clickAway={false}
      onOpenChange={onOpenChange}
      className="border-none rounded-sm"
      data-testid="streaming-optimized-label-toast"
      content={
        <div className="flex flex-row justify-between gap-5 items-center">
          <span className="dark:text-white text-baltic-sea">
            {t('streaming-optimized-label')}
          </span>
          <ButtonText
            color="transparent"
            className="!text-malachite"
            onClick={handleClick}
          >
            <span className="">{t('streaming-optimized-label-button')}</span>
          </ButtonText>
        </div>
      }
    />
  );
}
