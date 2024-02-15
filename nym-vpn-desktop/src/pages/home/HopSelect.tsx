import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { NodeHop, NodeLocation, isCountry } from '../../types';
import { useMainState } from '../../contexts';
import { MsIcon } from '../../ui';

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
        'relative',
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
          <div className="w-7 flex justify-center items-center">
            <img
              src={`./flags/${nodeLocation.code.toLowerCase()}.svg`}
              className="h-7 scale-90 pointer-events-none fill-current"
              alt={nodeLocation.code}
            />
          </div>
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
      <MsIcon icon="arrow_right" style="pointer-events-none" />
    </div>
  );
}
