import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { Button } from '@headlessui/react';
import { CardHeaderSwitch, CardNew, CardNewBody, Slider } from '../../../ui';
import { useMixnetTrafficConfig } from './context';

const BACKGROUND_COVER_TRAFFIC_RATE_LABELS = [
  'low',
  'balanced',
  'medium',
  'high',
] as const;

function BackgroundCoverSlider({
  value,
  setValue,
  disabled,
}: {
  value: number;
  setValue: (value: number) => void;
  disabled?: boolean;
}) {
  const { t } = useTranslation('settings');
  const { backgroundCoverItems } = useMixnetTrafficConfig();

  const items = backgroundCoverItems.map((item, index) => ({
    value: item.value,
    label: BACKGROUND_COVER_TRAFFIC_RATE_LABELS[index],
  }));

  return (
    <div className="w-full space-y-5">
      <p className="text-text-secondary text-sm whitespace-pre-line">
        {t('mixnet-tuning.background-cover.description')}
      </p>

      <div className="text-text-secondary flex justify-between text-sm">
        <span>{t('mixnet-tuning.slider.faster')}</span>
        <span>{t('mixnet-tuning.slider.anonymity')}</span>
      </div>
      <Slider
        className="px-2"
        value={value}
        onValueCommitted={setValue}
        min={0}
        max={3}
        step={1}
        disabled={disabled}
        ariaLabel={t('mixnet-tuning.background-cover.title')}
        labels={items.map((item, index) => (
          <Button
            onClick={() => setValue(index)}
            key={item.label}
            disabled={disabled}
            className={clsx('flex flex-col text-sm whitespace-nowrap', {
              'items-start': index === 0,
              'items-end': index === backgroundCoverItems.length - 1,
              'items-center':
                index !== 0 && index !== backgroundCoverItems.length - 1,
              'text-brand-primary': value === index,
              'text-text-secondary': value !== index,
            })}
          >
            <span className="whitespace-pre-line">
              {t(`mixnet-tuning.background-cover.${item.label}.label`)}
            </span>
          </Button>
        ))}
      />
    </div>
  );
}

export function BackgroundCoverCard() {
  const { t } = useTranslation('settings');

  const { state, updateField, backgroundCoverItems } = useMixnetTrafficConfig();

  const enabled = !state.disableBackgroundCoverTraffic;
  const toggle = () => updateField('disableBackgroundCoverTraffic', enabled);

  const setPoissonParameterForLoopCoverStream = (index: number) => {
    const item = backgroundCoverItems[index];
    if (item) {
      updateField('poissonParameterForLoopCoverStream', item.value);
    }
  };

  return (
    <CardNew>
      <CardHeaderSwitch
        checked={enabled}
        onClick={toggle}
        header={t('mixnet-tuning.background-cover.title')}
      />

      <CardNewBody className="pb-5">
        <BackgroundCoverSlider
          value={backgroundCoverItems.findIndex(
            (item) => item.value === state.poissonParameterForLoopCoverStream,
          )}
          setValue={setPoissonParameterForLoopCoverStream}
          disabled={!enabled}
        />
      </CardNewBody>
    </CardNew>
  );
}
