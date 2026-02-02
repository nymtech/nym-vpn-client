import clsx from 'clsx';
import SliderNew from '../../../ui/SliderNew';
import { SettingsMenuCardBig } from '../../../ui/index';
import { useMixnetTrafficConfig } from './context/index';

const MIXING_DELAY_LEVELS = [
  { label: 'Low', speed: '0ms' },
  { label: 'High', speed: '200ms' },
];

function MixingDelaySlider({
  value,
  setValue,
}: {
  value: number;
  setValue: (value: number) => void;
}) {
  return (
    <div className="w-full max-w-xl mt-5 space-y-5">
      <div className="flex justify-between text-sm text-iron dark:text-bombay">
        <span>Faster</span>
        <span>Max. anonymity</span>
      </div>

      <SliderNew
        className="px-2"
        value={value}
        defaultValue={25}
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
            <span className="whitespace-nowrap">{item.label}</span>
            <span className="whitespace-nowrap">{item.speed}</span>
          </div>
        ))}
      />
    </div>
  );
}

export function MixingDelayCard() {
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
      ? '⚠️  Timing protection is currently turned off. This could put your privacy at risk.'
      : 'Adjust timing delays to change how your packets are mixed.';

  return (
    <SettingsMenuCardBig
      header={
        <div className="w-full flex flex-row p-5 pb-0">
          <p className="text-left truncate text-base text-baltic-sea dark:text-white select-none">
            Mixing delays
          </p>
        </div>
      }
    >
      <p
        className={clsx('text-sm whitespace-pre-line', {
          'text-cheddar dark:text-king-nacho': value === 0,
          'text-iron dark:text-bombay': value !== 0,
        })}
      >
        {description}
      </p>

      <MixingDelaySlider value={value} setValue={setValue} />
    </SettingsMenuCardBig>
  );
}
