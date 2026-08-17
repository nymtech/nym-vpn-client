import { useCallback } from 'react';
import { Button } from '@headlessui/react';
import clsx from 'clsx';
import { UiGateway } from '../../../types/node';
import { nodeToFavorite } from '../../../types/favorites';
import { useNodeListState } from '../../../store/nodeListState';
import { ButtonIcon, MsIcon } from '../../../ui';
import FavoriteStar from '../FavoriteStar';
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
  /**
   * Show the country (and region, where applicable) alongside the city. Needed
   * wherever a row is shown outside its country grouping — search results and
   * the recents list — since nothing else supplies that context.
   */
  fullLocation?: boolean;
};

const GatewayItem = ({
  gateway,
  node,
  vpnMode,
  onSelect,
  onNodeDetails,
  quicLabel,
  fullLocation,
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
    if (fullLocation) {
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
      if (!htmlElement) return;
      if (focused?.type === 'gateway' && focused.key === gateway.id) {
        htmlElement.scrollIntoView({
          behavior: 'smooth',
          block: 'start',
        });
      }
    },
    [focused, gateway.id],
  );

  return (
    <div
      ref={scrollToGatewayRef}
      className={clsx(
        'flex flex-row items-center justify-between p-2 select-none',
        'hover:bg-surface-hair',
        'last:border-illustration-accent',
        'group-last/region:last:rounded-b-2xl',
        'border-surface-hair border-b',
      )}
    >
      <Button
        className="flex w-full items-center overflow-hidden pr-2 focus:outline-none"
        onClick={handleSelect}
      >
        <div
          className={clsx(
            'w-1 shrink-0 self-stretch rounded-r-sm',
            isSelected === node && 'bg-brand-primary',
            isSelected && isSelected !== node && 'bg-text-secondary',
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
        <MsIcon icon="smart_display" className="text-status-info" />
      )}
      <FavoriteStar
        favorite={nodeToFavorite(gateway)}
        isFavorite={gateway.isFavorite}
        hop={node}
      />
      <div className="flex items-center self-stretch p-2">
        <ButtonIcon
          color="chalk"
          icon="chevron_right"
          iconClassName="flex! items-center justify-center hover:text-brand-primary"
          onClick={() => onNodeDetails(gateway)}
        />
      </div>
    </div>
  );
};

export default GatewayItem;
