import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { CardNew, CardNewBody, CardNewHeader, Slider } from '../../../ui';
import { useMixnetTrafficConfig } from './context';

function MixingDelaySlider({
  value,
  setValue,
}: {
  value: number;
  setValue: (value: number) => void;
}) {
  const { t } = useTranslation('settings');
  const { mixingDelay } = useMixnetTrafficConfig();

  // endpoints are the daemon's allowed range
  const levels = [
    { key: 'low', ms: mixingDelay.minValue },
    { key: 'high', ms: mixingDelay.maxValue },
  ] as const;

  return (
    <div className="mt-5 w-full space-y-5">
      <div className="text-text-secondary flex justify-between text-sm">
        <span>{t('mixnet-tuning.slider.faster')}</span>
        <span>{t('mixnet-tuning.slider.anonymity')}</span>
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
        labels={levels.map((level, index) => (
          <div
            key={level.key}
            className={clsx('flex flex-col text-sm', {
              'items-start': index === 0,
              'items-end': index === levels.length - 1,
              'items-center': index !== 0 && index !== levels.length - 1,
            })}
          >
            <span className="whitespace-nowrap">
              {t(`mixnet-tuning.mixing-delay.${level.key}.label`)}
            </span>
            <span className="whitespace-nowrap">
              {t('mixnet-tuning.mixing-delay.ms-value', { value: level.ms })}
            </span>
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
            'text-status-warning': value === 0,
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
