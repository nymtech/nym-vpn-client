import { Button } from '@headlessui/react';
import clsx from 'clsx';
import { UiCountry, UiGateway } from '../../../contexts';
import { MsIcon } from '../../../ui';
import { NodeHop, VpnMode } from '../../../types';
import { getScoreIcon } from './util';

type GatewayRowProps = {
  gateway: UiGateway;
  onSelect: (gateway: UiGateway) => void;
  onNodeDetails: (node: UiGateway | UiCountry) => void;
  node: NodeHop;
  vpnMode: VpnMode;
};

const GatewayItem = ({
  gateway,
  node,
  vpnMode,
  onSelect,
  onNodeDetails,
}: GatewayRowProps) => {
  const { isSelected } = gateway;
  const scoreIcon = getScoreIcon(gateway, vpnMode);

  const handleSelect = () => {
    if (isSelected) {
      return;
    }
    onSelect(gateway);
  };

  const truncateId = (id: string) => {
    if (id.length < 10) {
      return id;
    }
    return `${id.slice(0, 5)}…${id.slice(-5)}`;
  };

  return (
    <div
      className={clsx(
        'flex flex-row justify-between items-center select-none',
        'hover:bg-mercury hover:dark:bg-mine-shaft',
      )}
      data-testid={`gateway-item-${gateway.id.substring(0, 8)}`}
      data-selected={isSelected ? isSelected : 'none'}
    >
      <Button
        className="flex items-center overflow-hidden w-full pr-2 focus:outline-none"
        onClick={handleSelect}
        data-testid={`gateway-select-button-${gateway.id.substring(0, 8)}`}
      >
        <div
          className={clsx(
            'w-1.5 rounded-r-sm shrink-0 self-stretch',
            isSelected === node && 'bg-malachite',
            isSelected && isSelected !== node && 'bg-iron',
          )}
          data-testid={`gateway-selection-indicator-${gateway.id.substring(0, 8)}`}
        />
        <div className="flex flex-row items-center p-2 gap-4 overflow-hidden">
          <div className="flex">
            <MsIcon
              className={clsx(scoreIcon[1], 'text-xl')}
              icon={scoreIcon[0]}
              data-testid={`gateway-score-icon-${gateway.id.substring(0, 8)}`}
            />
          </div>
          <div className="flex flex-col text-start overflow-hidden">
            <p
              className="truncate"
              data-testid={`gateway-name-${gateway.id.substring(0, 8)}`}
            >
              {gateway.name}
            </p>
            <p
              className="text-sm text-iron dark:text-bombay truncate"
              data-testid={`gateway-id-${gateway.id.substring(0, 8)}`}
            >
              {truncateId(gateway.id)}
            </p>
          </div>
        </div>
      </Button>
      <div className="flex py-2 self-stretch">
        <Button
          className={clsx(
            'w-16 flex justify-center items-center mr-3 shrink-0',
            'border-l-1 border-bombay dark:border-iron',
            'text-baltic-sea/80 dark:text-white/80',
            'hover:text-baltic-sea dark:hover:text-white',
            'focus:outline-none',
          )}
          onClick={() => onNodeDetails(gateway)}
          data-testid={`gateway-info-button-${gateway.id.substring(0, 8)}`}
        >
          <MsIcon
            icon="info"
            data-testid={`gateway-info-icon-${gateway.id.substring(0, 8)}`}
          />
        </Button>
      </div>
    </div>
  );
};

export default GatewayItem;
