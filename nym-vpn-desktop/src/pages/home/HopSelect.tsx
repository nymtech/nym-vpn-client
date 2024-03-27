import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { NodeHop, NodeLocation, isCountry } from '../../types';
import { useMainState } from '../../contexts';
import { FlagIcon, MsIcon, countryCode } from '../../ui';

interface HopSelectProps {
  nodeLocation: NodeLocation;
  onClick: () => void;
  nodeHop: NodeHop;
}

export default function HopSelect({
  nodeHop,
  nodeLocation,
  onClick,
}: HopSelectProps) {
  const { state, fastestNodeLocation } = useMainState();
  const { t } = useTranslation('home');

  return (
    <div
      className={clsx([
        state === 'Disconnected' ? 'cursor-pointer' : 'cursor-not-allowed',
        'w-full flex flex-row justify-between items-center py-3 px-4',
        'text-baltic-sea dark:text-mercury-pinkish',
        'border border-cement-feet dark:border-gun-powder rounded-lg',
        'hover:ring-4 ring-aluminium ring-opacity-35',
        'dark:ring-onyx dark:ring-opacity-65',
        'relative transition',
      ])}
      onKeyDown={onClick}
      role="presentation"
      onClick={onClick}
    >
      <div
        className={clsx([
          'absolute left-3 -top-2.5 px-1',
          'bg-blanc-nacre dark:bg-baltic-sea text-xs',
        ])}
      >
        {nodeHop === 'entry' ? t('first-hop') : t('last-hop')}
      </div>
      {isCountry(nodeLocation) && (
        <div className="flex flex-row items-center gap-3">
          <FlagIcon
            code={nodeLocation.code.toLowerCase() as countryCode}
            alt={nodeLocation.code}
          />
          <div className="text-base">{nodeLocation.name}</div>
        </div>
      )}
      {nodeLocation === 'Fastest' && (
        <div className="flex flex-row items-center gap-3">
          <div className="w-7 flex justify-center items-center">
            <MsIcon icon="bolt" />
          </div>
          <div className="text-base">{`${t('fastest', { ns: 'common' })} (${
            fastestNodeLocation.name
          })`}</div>
        </div>
      )}
      <MsIcon icon="arrow_right" className="pointer-events-none" />
    </div>
  );
}
