import { useCallback } from 'react';
import { Button } from '@headlessui/react';
import clsx from 'clsx';
import { UiGateway } from '../../../types/node';
import { useNodeListState } from '../../../store/nodeListState';
import { MsIcon } from '../../../ui';
import { NodeHop, VpnMode } from '../../../types';
import { useLang } from '../../../hooks';
import { countriesWithRegions } from '../../../constants';
import QuicTag from '../QuicTag';
import { ScoreIndicator } from '../ScoreIndicator';

type GatewayRowProps = {
  gateway: UiGateway;
  onSelect: (gateway: UiGateway) => void;
  onNodeDetails: (node: UiGateway) => void;
  node: NodeHop;
  vpnMode: VpnMode;
  quicLabel: boolean;
  inSearchResult?: boolean;
};

const GatewayItem = ({
  gateway,
  node,
  vpnMode,
  onSelect,
  onNodeDetails,
  quicLabel,
  inSearchResult,
}: GatewayRowProps) => {
  const { exit: exitNodeList, entry: entryNodeList } = useNodeListState();
  const focused =
    node === 'entry' ? entryNodeList.focused : exitNodeList.focused;

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

  const scrollToGatewayRef = useCallback(
    (htmlElement: HTMLDivElement) => {
      if (!htmlElement || inSearchResult) return;
      if (focused?.type === 'gateway' && focused.key === gateway.id) {
        htmlElement.scrollIntoView({
          behavior: 'smooth',
          block: 'start',
        });
      }
    },
    [focused, gateway.id, inSearchResult],
  );

  return (
    <div
      ref={scrollToGatewayRef}
      className={clsx(
        'flex flex-row items-center justify-between rounded-2xl select-none',
        'hover:bg-mercury hover:dark:bg-mine-shaft',
      )}
    >
      <Button
        className="flex w-full items-center overflow-hidden pr-2 focus:outline-none"
        onClick={handleSelect}
      >
        <div
          className={clsx(
            'w-1.5 shrink-0 self-stretch rounded-r-sm',
            isSelected === node && 'bg-malachite',
            isSelected && isSelected !== node && 'bg-iron',
          )}
        />
        <div className="flex flex-row items-center gap-4 overflow-hidden p-2">
          <div className="flex">
            <ScoreIndicator score={score} />
          </div>
          <div className="flex flex-col overflow-hidden text-start">
            <p className="text-text-primary truncate">{gateway.name}</p>
            <p className="text-text-secondary truncate text-sm">{location()}</p>
          </div>
        </div>
      </Button>
      {quicLabel && gateway.quic && <QuicTag />}
      {streamOptimized && (
        <MsIcon icon="smart_display" className="text-cornflower" />
      )}
      <div className="flex items-center self-stretch p-2">
        <Button
          className={clsx(
            'flex h-12 w-12 shrink-0 items-center justify-center rounded-full',
            'text-text-primary',
            'hover:bg-faded-lavender dark:hover:bg-charcoal',
            'focus:outline-none',
          )}
          onClick={() => onNodeDetails(gateway)}
        >
          <MsIcon icon="arrow_right" />
        </Button>
      </div>
    </div>
  );
};

export default GatewayItem;
