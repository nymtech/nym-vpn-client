import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { NodeHop, NodeLocation, isCountry } from '../../types';
import { useMainState, useNotifications } from '../../contexts';
import { FlagIcon, MsIcon, countryCode } from '../../ui';
import { useThrottle } from '../../hooks';

const snackbarThrottle = 6000;

interface HopSelectProps {
  nodeLocation: NodeLocation;
  onClick: () => void;
  nodeHop: NodeHop;
  disabled?: boolean;
}

export default function HopSelect({
  nodeHop,
  nodeLocation,
  onClick,
  disabled,
}: HopSelectProps) {
  const { fastestNodeLocation, state } = useMainState();
  const { t } = useTranslation('home');
  const { push } = useNotifications();

  const showSnack = useThrottle(
    async () => {
      let text = '';
      switch (state) {
        case 'Connected':
          text = t('snackbar-disabled-message.connected');
          break;
        case 'Connecting':
          text = t('snackbar-disabled-message.connecting');
          break;
        case 'Disconnecting':
          text = t('snackbar-disabled-message.disconnecting');
          break;
      }
      push({
        text,
        position: 'top',
      });
    },
    snackbarThrottle,
    [state],
  );

  const handleClick = () => {
    if (disabled) {
      showSnack();
    } else {
      onClick();
    }
  };

  return (
    <div
      className={clsx([
        'w-full flex flex-row justify-between items-center py-3 px-4',
        'text-baltic-sea dark:text-mercury-pinkish',
        'border border-cement-feet dark:border-gun-powder rounded-lg',
        'hover:border-baltic-sea hover:ring-baltic-sea',
        'dark:hover:border-mercury-pinkish dark:hover:ring-mercury-pinkish',
        'relative transition select-none cursor-default',
      ])}
      onKeyDown={handleClick}
      role="presentation"
      onClick={handleClick}
    >
      <div
        className={clsx([
          'absolute left-3 -top-2.5 px-1',
          'bg-blanc-nacre dark:bg-baltic-sea text-xs',
          disabled && 'cursor-default',
        ])}
      >
        {nodeHop === 'entry' ? t('first-hop') : t('last-hop')}
      </div>
      {isCountry(nodeLocation) && (
        <div className="flex flex-row items-center gap-3 overflow-hidden">
          <FlagIcon
            code={nodeLocation.code.toLowerCase() as countryCode}
            alt={nodeLocation.code}
          />
          <div
            className={clsx([
              'text-base truncate',
              disabled && 'cursor-default',
            ])}
          >
            {nodeLocation.name}
          </div>
        </div>
      )}
      {nodeLocation === 'Fastest' && (
        <div className="flex flex-row items-center gap-3">
          <div className="w-7 flex justify-center items-center">
            <MsIcon icon="bolt" />
          </div>
          <div
            className={clsx([
              'text-base truncate',
              disabled && 'cursor-default',
            ])}
          >{`${t('fastest', { ns: 'common' })} (${
            fastestNodeLocation.name
          })`}</div>
        </div>
      )}
      <MsIcon icon="arrow_right" className="pointer-events-none" />
    </div>
  );
}
