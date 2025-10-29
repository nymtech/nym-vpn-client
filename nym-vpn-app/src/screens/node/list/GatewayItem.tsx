import { Button } from '@headlessui/react';
import clsx from 'clsx';
import { UiGateway } from '../../../contexts';
import { MsIcon } from '../../../ui';
import { NodeHop, VpnMode } from '../../../types';
import { useLang } from '../../../hooks';
import { countriesWithRegions } from '../../../constants';
import QuicTag from '../QuicTag';
import { getScoreIcon } from './util';

type GatewayRowProps = {
  ref?: React.Ref<HTMLDivElement>;
  gateway: UiGateway;
  onSelect: (gateway: UiGateway) => void;
  onNodeDetails: (node: UiGateway) => void;
  node: NodeHop;
  vpnMode: VpnMode;
  quicLabel: boolean;
  inSearchResult?: boolean;
};

const GatewayItem = ({
  ref,
  gateway,
  node,
  vpnMode,
  onSelect,
  onNodeDetails,
  quicLabel,
  inSearchResult,
}: GatewayRowProps) => {
  const { isSelected } = gateway;
  const scoreIcon = getScoreIcon(gateway, vpnMode);
  const { getCountryName } = useLang();

  const handleSelect = () => {
    if (isSelected) {
      return;
    }
    onSelect(gateway);
  };

  const location = () => {
    if (inSearchResult) {
      const countryName =
        getCountryName(gateway.country.code) || gateway.country.name;
      if (countriesWithRegions.includes(gateway.country.code)) {
        return `${gateway.location.city}, ${gateway.location.region}, ${countryName}`;
      }
      return `${gateway.location.city}, ${countryName}`;
    }
    return gateway.location.city;
  };

  return (
    <div
      ref={ref}
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
            <p className="text-sm text-iron dark:text-bombay truncate">
              {location()}
            </p>
          </div>
        </div>
      </Button>
      {quicLabel && gateway.quic && <QuicTag />}
      <div className="flex py-2 self-stretch items-center">
        {gateway.asn?.type === 'residential' && (
          <MsIcon
            icon="smart_display"
            className="font-icon text-2xl select-none text-cornflower"
          />
        )}
        <Button
          className={clsx(
            'w-14 flex justify-center items-center mr-3 shrink-0',
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
