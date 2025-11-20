import { Button } from '@headlessui/react';
import clsx from 'clsx';
import { UiGateway } from '../../../contexts';
import { MsIcon } from '../../../ui';
import { NodeHop, VpnMode } from '../../../types';
import { useLang } from '../../../hooks';
import { countriesWithRegions } from '../../../constants';
import QuicTag from '../QuicTag';
import { ScoreIndicator } from '../ScoreIndicator';

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
  const score = vpnMode === 'mixnet' ? gateway.mxScore : gateway.wgScore;
  const { getCountryName } = useLang();
  const streamOptimized =
    node === 'exit' && gateway.asn?.type === 'residential';

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
            <ScoreIndicator score={score} />
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
      {streamOptimized && (
        <MsIcon icon="smart_display" className="text-cornflower" />
      )}
      <div className="flex py-2 self-stretch items-center">
        <Button
          className={clsx(
            'w-6 h-6 flex justify-center items-center mr-3 shrink-0',
            'text-baltic-sea dark:text-white',
            'hover:bg-faded-lavender dark:hover:bg-charcoal rounded-sm',
            'focus:outline-none',
          )}
          onClick={() => onNodeDetails(gateway)}
          data-testid={`gateway-info-button-${gateway.id.substring(0, 8)}`}
        >
          <MsIcon
            icon="arrow_right"
            data-testid={`gateway-info-icon-${gateway.id.substring(0, 8)}`}
          />
        </Button>
      </div>
    </div>
  );
};

export default GatewayItem;
