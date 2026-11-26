import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { Button } from '@headlessui/react';
import { CardHeaderSwitch, CardNew, CardNewBody, Slider } from '../../../ui';
import { useMixnetTrafficConfig } from './context';

function ContinuousTrafficSlider({
  value,
  setValue,
  disabled,
}: {
  value: number;
  setValue: (value: number) => void;
  disabled?: boolean;
}) {
  const { t } = useTranslation('settings');
  const { continuousItems: items } = useMixnetTrafficConfig();

  // Slowest rate (0.7 Mbps) on the left, fastest (2 Mbps) on the right: a higher
  // rate is a denser cover stream, so "+ Anonymity" sits on the right as on the
  // other mixnet tuning sliders.
  return (
    <div className="mt-0 w-full space-y-5">
      <p
        className={clsx('text-sm whitespace-pre-line', {
          'text-status-warning': disabled,
          'text-text-secondary': !disabled,
        })}
      >
        {disabled
          ? t('mixnet-tuning.continuous-traffic.warning')
          : t('mixnet-tuning.continuous-traffic.description')}
      </p>

      <div className="text-text-secondary flex justify-between text-sm">
        <span>{t('mixnet-tuning.slider.less-data')}</span>
        <span>{t('mixnet-tuning.slider.anonymity')}</span>
      </div>
      <Slider
        className="px-2"
        value={value}
        onChange={setValue}
        min={0}
        max={2}
        step={1}
        disabled={disabled}
        ariaLabel={t('mixnet-tuning.continuous-traffic.title')}
        labels={items.map((item, index) => (
          <Button
            key={item.value}
            disabled={disabled}
            className={clsx('flex flex-col text-sm', {
              'text-brand-primary': value === index,
              'text-text-secondary': value !== index,
              'items-start': index === 0,
              'items-end': index === items.length - 1,
              'items-center': index !== 0 && index !== items.length - 1,
            })}
            onClick={() => setValue(index)}
          >
            <span className="whitespace-nowrap">{item.label}</span>
          </Button>
        ))}
      />
    </div>
  );
}

export function ContinuousTrafficCard() {
  const { t } = useTranslation('settings');

  const { state, updateField, continuousItems } = useMixnetTrafficConfig();

  const enabled = !state.disablePoissonRate;
  const toggle = () => updateField('disablePoissonRate', enabled);

  const setMessageSendingAverageDelay = (index: number) => {
    const item = continuousItems[index];
    if (item) {
      updateField('messageSendingAverageDelay', item.value);
    }
  };

  return (
    <CardNew>
      <CardHeaderSwitch
        checked={enabled}
        onClick={toggle}
        header={t('mixnet-tuning.continuous-traffic.title')}
      />

      <CardNewBody className="pb-5">
        <ContinuousTrafficSlider
          value={continuousItems.findIndex(
            (item) => item.value === state.messageSendingAverageDelay,
          )}
          setValue={setMessageSendingAverageDelay}
          disabled={!enabled}
        />
      </CardNewBody>
    </CardNew>
  );
}
