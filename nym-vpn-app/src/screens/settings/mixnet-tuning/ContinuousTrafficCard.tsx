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
  const { continuousItems } = useMixnetTrafficConfig();

  // Reversed so the faster (2 Mbps) end sits on the LEFT, matching the "Faster" label.
  const items = [...continuousItems].reverse();

  return (
    <div className="mt-0 w-full space-y-5">
      <p className="text-text-secondary text-sm whitespace-pre-line">
        {t('mixnet-tuning.continuous-traffic.description')}
      </p>

      <div className="text-text-secondary flex justify-between text-sm">
        <span>{t('mixnet-tuning.slider.faster')}</span>
        <span>{t('mixnet-tuning.slider.slower')}</span>
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

  // Reversed to match the slider (faster / 2 Mbps on the left).
  const reversedItems = [...continuousItems].reverse();

  const enabled = !state.disablePoissonRate;
  const toggle = () => updateField('disablePoissonRate', enabled);

  const setMessageSendingAverageDelay = (index: number) => {
    const item = reversedItems[index];
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
        subheader={
          enabled ? undefined : t('mixnet-tuning.continuous-traffic.warning')
        }
        subheaderColor="king-nacho"
      />

      <CardNewBody className="pb-5">
        <ContinuousTrafficSlider
          value={reversedItems.findIndex(
            (item) => item.value === state.messageSendingAverageDelay,
          )}
          setValue={setMessageSendingAverageDelay}
          disabled={!enabled}
        />
      </CardNewBody>
    </CardNew>
  );
}
