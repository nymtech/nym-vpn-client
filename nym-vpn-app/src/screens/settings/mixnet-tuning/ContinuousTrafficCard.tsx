import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { CardSwitch, SettingsMenuCardBig, Slider } from '../../../ui';
import { useMixnetTrafficConfig } from './context';

const BACKGROUND_COVER_TRAFFIC_RATE_LEVELS: {
  label: 'base' | 'balanced' | 'medium' | 'high';
  speed: string;
}[] = [
  { label: 'base', speed: '5 pckt/s' },
  { label: 'balanced', speed: '5x' },
  { label: 'medium', speed: '10x' },
  { label: 'high', speed: '20x' },
];

function BackgroundCoverTrafficRateSlider({
  value,
  setValue,
}: {
  value: number;
  setValue: (value: number) => void;
}) {
  const { t } = useTranslation('settings');

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
        labels={BACKGROUND_COVER_TRAFFIC_RATE_LEVELS.map((item, index) => (
          <button
            onClick={() => setValue(index)}
            key={item.label}
            className={clsx('flex flex-col text-sm whitespace-nowrap ', {
              'items-start': index === 0,
              'items-end':
                index === BACKGROUND_COVER_TRAFFIC_RATE_LEVELS.length - 1,
              'items-center':
                index !== 0 &&
                index !== BACKGROUND_COVER_TRAFFIC_RATE_LEVELS.length - 1,
              'text-baltic-sea dark:text-white': value === index,
              'text-iron dark:text-bombay': value !== index,
            })}
          >
            <span className="whitespace-nowrap">
              {t(
                `mixnet-tuning.continuous-traffic.background-cover-traffic.${item.label}.label`,
              )}
            </span>
            <span className="whitespace-nowrap">{item.speed}</span>
          </button>
        ))}
      />
    </div>
  );
}

const CONTINUOUS_LEVELS: {
  label: 'low' | 'balanced' | 'high';
  speed: string;
}[] = [
  { label: 'low', speed: '0.7 Mbps' },
  { label: 'balanced', speed: '1 Mbps' },
  { label: 'high', speed: '2 Mbps' },
];

// Slider index → Config value mappings
// Array index = slider value, array value = config value
const CONTINUOUS_TRAFFIC_VALUES = [30, 20, 10] as const;
const BACKGROUND_COVER_TRAFFIC_VALUES = [60, 40, 20, 10] as const;

// Reverse lookup maps: Config value → Slider index
// Generated from arrays to maintain single source of truth
const CONTINUOUS_TRAFFIC_TO_SLIDER = new Map<number, number>(
  (CONTINUOUS_TRAFFIC_VALUES as readonly number[]).map((value, index) => [
    value,
    index,
  ]),
);
const BACKGROUND_COVER_TRAFFIC_TO_SLIDER = new Map<number, number>(
  (BACKGROUND_COVER_TRAFFIC_VALUES as readonly number[]).map((value, index) => [
    value,
    index,
  ]),
);

// Helper functions to convert between slider and config values
const sliderToContinuousTrafficValue = (sliderValue: number): number =>
  CONTINUOUS_TRAFFIC_VALUES[sliderValue] ?? CONTINUOUS_TRAFFIC_VALUES[0];

const continuousTrafficValueToSlider = (configValue: number): number =>
  CONTINUOUS_TRAFFIC_TO_SLIDER.get(configValue) ?? 0;

const sliderToBackgroundCoverTrafficValue = (sliderValue: number): number =>
  BACKGROUND_COVER_TRAFFIC_VALUES[sliderValue] ??
  BACKGROUND_COVER_TRAFFIC_VALUES[0];

const backgroundCoverTrafficValueToSlider = (configValue: number): number =>
  BACKGROUND_COVER_TRAFFIC_TO_SLIDER.get(configValue) ?? 0;

function ContinuousTrafficSlider({
  value,
  setValue,
}: {
  value: number;
  setValue: (value: number) => void;
}) {
  const { t } = useTranslation('settings');

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
        onValueCommitted={setValue}
        min={0}
        max={2}
        step={1}
        labels={CONTINUOUS_LEVELS.map((item, index) => (
          <button
            key={item.label}
            className={clsx('flex flex-col text-sm', {
              'text-baltic-sea dark:text-white': value === index,
              'text-iron dark:text-bombay': value !== index,
              'items-start': index === 0,
              'items-end': index === CONTINUOUS_LEVELS.length - 1,
              'items-center':
                index !== 0 && index !== CONTINUOUS_LEVELS.length - 1,
            })}
            onClick={() => setValue(index)}
          >
            <span className="whitespace-nowrap">
              {t(
                `mixnet-tuning.continuous-traffic.continuous.${item.label}.label`,
              )}
            </span>
            <span className="whitespace-nowrap">{item.speed}</span>
          </button>
        ))}
      />
    </div>
  );
}

export function ContinuousTrafficCard() {
  const { t } = useTranslation('settings');

  const { state, dispatch } = useMixnetTrafficConfig();

  const enabled = !state.disableBackgroundCoverTraffic;
  const setEnabled = (enabled: boolean) =>
    dispatch({
      type: 'update-field',
      field: 'disableBackgroundCoverTraffic',
      value: enabled,
    });

  const setMessageSendingAverageDelay = (value: number) =>
    dispatch({
      type: 'update-field',
      field: 'messageSendingAverageDelay',
      value,
    });

  const setPoissonParameterForLoopCoverStream = (value: number) =>
    dispatch({
      type: 'update-field',
      field: 'poissonParameterForLoopCoverStream',
      value,
    });

  return (
    <SettingsMenuCardBig
      header={
        <CardSwitch
          checked={enabled}
          onClick={() => setEnabled(enabled)}
          header={t('mixnet-tuning.continuous-traffic.continuous.title')}
        />
      }
    >
      {enabled && (
        <ContinuousTrafficSlider
          value={continuousTrafficValueToSlider(
            state.messageSendingAverageDelay,
          )}
          setValue={(value) =>
            setMessageSendingAverageDelay(sliderToContinuousTrafficValue(value))
          }
        />
      )}
      {!enabled && (
        <BackgroundCoverTrafficRateSlider
          value={backgroundCoverTrafficValueToSlider(
            state.poissonParameterForLoopCoverStream,
          )}
          setValue={(value) =>
            setPoissonParameterForLoopCoverStream(
              sliderToBackgroundCoverTrafficValue(value),
            )
          }
        />
      )}
    </SettingsMenuCardBig>
  );
}
