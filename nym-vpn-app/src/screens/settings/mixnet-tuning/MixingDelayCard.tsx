import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { CardNew, CardNewBody, CardNewHeader, Slider } from '../../../ui';
import {
  DEFAULT_MIXNET_TRAFFIC_CONFIG,
  useMixnetTrafficConfig,
} from './context';

const MIXING_DELAY_LEVELS: { label: 'low' | 'high'; speed: string }[] = [
  { label: 'low', speed: '0ms' },
  { label: 'high', speed: '200ms' },
];

function MixingDelaySlider({
  value,
  setValue,
}: {
  value: number | null;
  setValue: (value: number) => void;
}) {
  const { t } = useTranslation('settings');

  return (
    <div className="w-full mt-5 space-y-5">
      <div className="flex justify-between text-sm text-iron dark:text-bombay">
        <span>{t('mixnet-tuning.mixing-delay.faster')}</span>
        <span>{t('mixnet-tuning.mixing-delay.max-anonymity')}</span>
      </div>

      <Slider
        className="px-2"
        value={value ?? DEFAULT_MIXNET_TRAFFIC_CONFIG.averagePacketDelay!}
        defaultValue={DEFAULT_MIXNET_TRAFFIC_CONFIG.averagePacketDelay!}
        onChange={setValue}
        min={0}
        max={200}
        step={1}
        valueIndicator
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
  const { state, dispatch } = useMixnetTrafficConfig();

  const value = state.averagePacketDelay;
  const setValue = (value: number) =>
    dispatch({
      type: 'update-field',
      field: 'averagePacketDelay',
      value,
    });

  const description =
    value === 0
      ? t('mixnet-tuning.mixing-delay.warning')
      : t('mixnet-tuning.mixing-delay.description');

  return (
    <CardNew>
      <CardNewHeader>
        <p className="text-left truncate text-base text-baltic-sea dark:text-white select-none">
          {t('mixnet-tuning.mixing-delay.title')}
        </p>
      </CardNewHeader>
      <CardNewBody className="pb-5">
        <p
          className={clsx('text-sm whitespace-pre-line', {
            'text-cheddar dark:text-king-nacho': value === 0,
            'text-iron dark:text-bombay': value !== 0,
          })}
        >
          {description}
        </p>

        <MixingDelaySlider value={value} setValue={setValue} />
      </CardNewBody>
    </CardNew>
  );
}
