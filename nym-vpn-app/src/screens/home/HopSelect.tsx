import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { useNavigate } from 'react-router';
import { Button } from '@headlessui/react';
import { useShallow } from 'zustand/react/shallow';
import {
  Gateway,
  NodeHop,
  SelectedNode,
  isCountry,
  isGateway,
  isRegion,
} from '../../types';
import { MsIcon, countryCode } from '../../ui';
import { useLang } from '../../hooks';
import { useLookupGw } from '../../contexts';
import { useAppStore } from '../../store';
import { countriesWithRegions } from '../../constants';
import { routes } from '../../router';
import { isBridgeMode, regionToCountryCode, useActionToast } from './util';
import {
  SelectedNodeDisplay,
  SelectedNodeDisplayProps,
} from './SelectedNodeDisplay';

type HopSelectProps = {
  node: SelectedNode;
  gatewayId: string | null;
  onClick: () => void;
  nodeHop: NodeHop;
  disabled?: boolean;
};

export default function HopSelect({
  nodeHop,
  node,
  gatewayId,
  onClick,
  disabled,
}: HopSelectProps) {
  const { backendFlags, vpnMode, tunnel, connectingState } = useAppStore(
    useShallow((s) => ({
      backendFlags: s.backendFlags,
      vpnMode: s.vpnMode,
      tunnel: s.tunnel,
      connectingState: s.connectingState,
    })),
  );
  const lookupGw = useLookupGw();
  const { t } = useTranslation('home');
  const { getCountryName } = useLang();
  const toast = useActionToast('node-select');
  const navigate = useNavigate();
  const quicConnection =
    isBridgeMode(tunnel?.data) || isBridgeMode(connectingState?.tunnel);
  const quicTag =
    vpnMode === 'wg' &&
    nodeHop === 'entry' &&
    backendFlags.quic &&
    quicConnection;

  const handleClick = () => {
    if (disabled) {
      toast();
    } else {
      onClick();
    }
  };

  const handleDetailsClick = () => {
    if (!gateway) return;

    navigate(routes.nodeDetails, {
      state: { gateway, hop: nodeHop, resetScroll: true },
    });
  };

  const nodeData = (
    selected: SelectedNode,
    gateway: Gateway | null,
  ): SelectedNodeDisplayProps => {
    if (selected === 'random') {
      return {
        name: t('random', { ns: 'common' }),
        animate: false,
        showQuic: Boolean(quicTag && gateway?.quic),
        showStreamOptimized:
          nodeHop === 'exit' && gateway?.asn?.type === 'residential',
        showFastest: node === 'random' && !gateway?.country?.code,
      };
    }
    if (isCountry(selected)) {
      return getLocationInfo(selected.country.code, gateway);
    }
    if (isRegion(selected)) {
      return getLocationInfo(
        // TODO handle this better, ie. vpnd should provide country code along with region
        regionToCountryCode(selected.region) || 'US',
        gateway,
        selected.region,
      );
    }
    if (isGateway(selected)) {
      return getGatewayInfo(selected.gateway.id, gateway);
    }
    return {
      name: t('random', { ns: 'common' }),
      animate: false,
      showQuic: Boolean(quicTag && gateway?.quic),
      showStreamOptimized:
        nodeHop === 'exit' && gateway?.asn?.type === 'residential',
      showFastest: node === 'random' && !gateway?.country?.code,
    };
  };

  const getLocationInfo = (
    countryCode: string,
    gateway: Gateway | null,
    region?: string,
  ): SelectedNodeDisplayProps => {
    let location = getCountryName(countryCode) || countryCode;
    let subInfo = null;
    if (region && region.length > 0) {
      location = `${location}, ${region}`;
    }
    if (gateway) {
      const components = [];
      if (gateway.location.city.length > 0) {
        components.push(gateway.location.city);
      }
      if (!region && countriesWithRegions.includes(countryCode)) {
        components.push(gateway.location.region);
      }
      subInfo = `${components.join(', ')} (${gateway.name})`;
    }

    return {
      countryCode: countryCode.toLowerCase() as countryCode,
      name: location,
      subInfo,
      animate: true,
      showQuic: Boolean(quicTag && gateway?.quic),
      showStreamOptimized:
        nodeHop === 'exit' && gateway?.asn?.type === 'residential',
      showFastest: node === 'random' && !gateway?.country?.code,
    };
  };

  const getGatewayInfo = (
    id: string,
    gateway: Gateway | null,
  ): SelectedNodeDisplayProps => {
    if (!gateway) {
      return {
        name: id,
      };
    }

    const { country, location, name } = gateway;
    const components = [];
    if (location.city.length > 0) {
      components.push(location.city);
    }
    if (
      countriesWithRegions.includes(country.code) &&
      location.region.length > 0
    ) {
      components.push(location.region);
    }
    components.push(getCountryName(country.code) || country.name);

    return {
      countryCode: country.code.toLowerCase() as countryCode,
      name,
      subInfo: components.join(', '),
      showQuic: Boolean(quicTag && gateway?.quic),
      showStreamOptimized:
        nodeHop === 'exit' && gateway?.asn?.type === 'residential',
      showFastest: node === 'random' && !gateway?.country?.code,
    };
  };

  const gateway = useMemo(() => {
    if (node === 'random') {
      return null;
    }
    if (isGateway(node)) {
      return lookupGw(node.gateway.id, nodeHop);
    } else if (gatewayId) {
      return lookupGw(gatewayId, nodeHop);
    }
    return null;
  }, [gatewayId, lookupGw, nodeHop, node]);

  return (
    <div
      className={clsx([
        'flex h-[3.75rem] w-full flex-row items-center justify-between',
        'text-text-primary',
        'border-text-tertiary dark:border-text-secondary rounded-lg border',
        'relative cursor-default transition select-none',
        disabled && 'opacity-50',
      ])}
      role="presentation"
    >
      <div
        className={clsx([
          'absolute -top-2 left-3 px-1',
          'bg-surface-bg text-xs',
          disabled && 'cursor-default',
        ])}
      >
        {nodeHop === 'entry' ? t('first-hop') : t('last-hop')}
      </div>

      <Button
        className={clsx([
          'flex h-full flex-1 items-center justify-center overflow-hidden rounded-none rounded-l-lg py-3 ps-4',
          !disabled && 'hover:text-text-primary/80 dark:hover:text-white/80',
        ])}
        onClick={handleClick}
        onKeyDown={handleClick}
      >
        <SelectedNodeDisplay {...nodeData(node, gateway)} disabled={disabled} />
      </Button>
      {!!gateway && (
        <Button
          className={clsx(
            'my-2 me-2 flex h-11 w-11 items-center justify-center rounded-full',
            !disabled && 'hover:bg-surface-elev dark:hover:bg-surface-elev',
          )}
          onClick={handleDetailsClick}
          onKeyDown={handleDetailsClick}
        >
          <MsIcon
            icon="arrow_right"
            className="text-text-primary leading-none"
          />
        </Button>
      )}
    </div>
  );
}
