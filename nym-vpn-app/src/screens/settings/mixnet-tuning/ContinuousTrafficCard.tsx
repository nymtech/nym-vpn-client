import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { useMemo } from 'react';
import { Button } from '@headlessui/react';
import { CardHeaderSwitch, CardNew, CardNewBody, Slider } from '../../../ui';
import { useMixnetTrafficConfig } from './context';

const BACKGROUND_COVER_TRAFFIC_RATE_LABELS = [
  'base',
  'balanced',
  'medium',
  'high',
] as const;

function BackgroundCoverTrafficRateSlider({
  value,
  setValue,
}: {
  value: number;
  setValue: (value: number) => void;
}) {
  const { t } = useTranslation('settings');
  const { backgroundCoverItems } = useMixnetTrafficConfig();

  const items = useMemo(
    () =>
      backgroundCoverItems.map((item, index) => ({
        value: item.value,
        label: BACKGROUND_COVER_TRAFFIC_RATE_LABELS[index],
      })),
    [backgroundCoverItems],
  );

  return (
    <div className="w-full space-y-5">
      <p className="text-sm text-cheddar dark:text-king-nacho whitespace-pre-line">
        {t('mixnet-tuning.continuous-traffic.background-cover-traffic.warning')}
      </p>

      <p className="truncate text-base text-baltic-sea dark:text-white select-none">
        {t('mixnet-tuning.continuous-traffic.background-cover-traffic.title')}
      </p>
      <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
        {t(
          'mixnet-tuning.continuous-traffic.background-cover-traffic.description',
        )}
      </p>
      <div className="flex justify-between text-sm text-iron dark:text-bombay">
        <span>
          {t(
            'mixnet-tuning.continuous-traffic.background-cover-traffic.use-less-battery-and-data',
          )}
        </span>
        <span>
          {t(
            'mixnet-tuning.continuous-traffic.background-cover-traffic.max-anonymity',
          )}
        </span>
      </div>
      <Slider
        className="px-2"
        value={value}
        onValueCommitted={setValue}
        min={0}
        max={3}
        step={1}
        ariaLabel={t(
          'mixnet-tuning.continuous-traffic.background-cover-traffic.title',
        )}
        labels={items.map((item, index) => (
          <Button
            onClick={() => setValue(index)}
            key={item.label}
            className={clsx('flex flex-col text-sm whitespace-nowrap ', {
              'items-start': index === 0,
              'items-end': index === backgroundCoverItems.length - 1,
              'items-center':
                index !== 0 && index !== backgroundCoverItems.length - 1,
              'text-baltic-sea dark:text-white': value === index,
              'text-iron dark:text-bombay': value !== index,
            })}
          >
            <span className="whitespace-pre-line">
              {t(
                `mixnet-tuning.continuous-traffic.background-cover-traffic.${item.label}.label`,
              )}
            </span>
          </Button>
        ))}
      />
    </div>
  );
}

const CONTINUOUS_TRAFFIC_SENDING_RATE_LABELS = [
  'low',
  'balanced',
  'high',
] as const;

function ContinuousTrafficSlider({
  value,
  setValue,
}: {
  value: number;
  setValue: (value: number) => void;
}) {
  const { t } = useTranslation('settings');
  const { continuousItems } = useMixnetTrafficConfig();

  const items = useMemo(
    () =>
      continuousItems.map((item, index) => ({
        value: item.value,
        label: CONTINUOUS_TRAFFIC_SENDING_RATE_LABELS[index],
        speed: item.label,
      })),
    [continuousItems],
  );
  return (
    <div className="w-full mt-0 space-y-5">
      <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
        {t('mixnet-tuning.continuous-traffic.continuous.title')}
      </p>

      <div className="flex justify-between text-sm text-iron dark:text-bombay">
        <span>
          {t(
            'mixnet-tuning.continuous-traffic.continuous.use-less-battery-and-data',
          )}
        </span>
        <span>
          {t('mixnet-tuning.continuous-traffic.continuous.max-anonymity')}
        </span>
      </div>
      <Slider
        className="px-2"
        value={value}
        onChange={setValue}
        min={0}
        max={2}
        step={1}
        ariaLabel={t('mixnet-tuning.continuous-traffic.continuous.title')}
        labels={items.map((item, index) => (
          <Button
            key={item.label}
            className={clsx('flex flex-col text-sm', {
              'text-baltic-sea dark:text-white': value === index,
              'text-iron dark:text-bombay': value !== index,
              'items-start': index === 0,
              'items-end': index === continuousItems.length - 1,
              'items-center':
                index !== 0 && index !== continuousItems.length - 1,
            })}
            onClick={() => setValue(index)}
          >
            <span className="whitespace-nowrap">
              {t(
                `mixnet-tuning.continuous-traffic.continuous.${item.label}.label`,
              )}
            </span>
            <span className="whitespace-nowrap">{item.speed}</span>
          </Button>
        ))}
      />
    </div>
  );
}

export function ContinuousTrafficCard() {
  const { t } = useTranslation('settings');

  const { state, updateField, continuousItems, backgroundCoverItems } =
    useMixnetTrafficConfig();

  const enabled = !state.disableBackgroundCoverTraffic;
  const setEnabled = (enabled: boolean) =>
    updateField('disableBackgroundCoverTraffic', enabled);

  const setMessageSendingAverageDelay = (index: number) => {
    const item = continuousItems[index];
    if (item) {
      updateField('messageSendingAverageDelay', item.value);
    }
  };

  const setPoissonParameterForLoopCoverStream = (index: number) => {
    const item = backgroundCoverItems[index];
    if (item) {
      updateField('poissonParameterForLoopCoverStream', item.value);
    }
  };

  return (
    <div className="flex flex-col gap-10">
      <CardNew>
        <CardHeaderSwitch
          checked={enabled}
          onClick={() => setEnabled(enabled)}
          header={t('mixnet-tuning.continuous-traffic.title')}
        />

        <CardNewBody className="pb-5">
          {enabled && (
            <ContinuousTrafficSlider
              value={continuousItems.findIndex(
                (item) => item.value === state.messageSendingAverageDelay,
              )}
              setValue={(index) => setMessageSendingAverageDelay(index)}
            />
          )}
          {!enabled && (
            <BackgroundCoverTrafficRateSlider
              value={backgroundCoverItems.findIndex(
                (item) =>
                  item.value === state.poissonParameterForLoopCoverStream,
              )}
              setValue={(index) => setPoissonParameterForLoopCoverStream(index)}
            />
          )}
        </CardNewBody>
      </CardNew>
    </div>
  );
}
