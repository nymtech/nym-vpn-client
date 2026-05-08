import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { CardNew, CardNewBody, CardNewHeader, Slider } from '../../../ui';
import { useMixnetTrafficConfig } from './context';

const MIXING_DELAY_LEVELS: { label: 'low' | 'high'; speed: string }[] = [
  { label: 'low', speed: '0ms' },
  { label: 'high', speed: '200ms' },
];

function MixingDelaySlider({
  value,
  setValue,
}: {
  value: number;
  setValue: (value: number) => void;
}) {
  const { t } = useTranslation('settings');
  const { mixingDelay } = useMixnetTrafficConfig();

  return (
    <div className="mt-5 w-full space-y-5">
      <div className="text-text-secondary flex justify-between text-sm">
        <span>{t('mixnet-tuning.mixing-delay.faster')}</span>
        <span>{t('mixnet-tuning.mixing-delay.max-anonymity')}</span>
      </div>

      <Slider
        className="px-2"
        value={value}
        onChange={setValue}
        min={mixingDelay.minValue}
        max={mixingDelay.maxValue}
        step={1}
        valueIndicator
        ariaLabel={t('mixnet-tuning.mixing-delay.title')}
        labels={MIXING_DELAY_LEVELS.map((item, index) => (
          <div
            key={item.label}
            className={clsx('flex flex-col text-sm', {
              'items-start': index === 0,
              'items-end': index === MIXING_DELAY_LEVELS.length - 1,
              'items-center':
                index !== 0 && index !== MIXING_DELAY_LEVELS.length - 1,
            })}
          >
            <span className="whitespace-nowrap">
              {t(`mixnet-tuning.mixing-delay.${item.label}.label`)}
            </span>
            <span className="whitespace-nowrap">{item.speed}</span>
          </div>
        ))}
      />
    </div>
  );
}

export function MixingDelayCard() {
  const { t } = useTranslation('settings');
  const { state, updateField } = useMixnetTrafficConfig();

  const value = state.averagePacketDelay;
  const setValue = (value: number) => updateField('averagePacketDelay', value);

  const description =
    value === 0
      ? t('mixnet-tuning.mixing-delay.warning')
      : t('mixnet-tuning.mixing-delay.description');

  return (
    <CardNew>
      <CardNewHeader>
        <p className="text-text-primary truncate text-left text-base select-none">
          {t('mixnet-tuning.mixing-delay.title')}
        </p>
      </CardNewHeader>
      <CardNewBody className="pb-5">
        <p
          className={clsx('text-sm whitespace-pre-line', {
            'text-cheddar dark:text-king-nacho': value === 0,
            'text-text-secondary': value !== 0,
          })}
        >
          {description}
        </p>

        <MixingDelaySlider value={value} setValue={setValue} />
      </CardNewBody>
    </CardNew>
  );
}
