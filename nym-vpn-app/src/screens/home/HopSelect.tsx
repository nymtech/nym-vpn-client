import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { AnimatePresence, motion } from 'motion/react';
import clsx from 'clsx';
import { useNavigate } from 'react-router';
import { Button } from '@headlessui/react';
import {
  Gateway,
  NodeHop,
  SelectedNode,
  isCountry,
  isGateway,
  isRegion,
} from '../../types';
import { FlagIcon, MsIcon, countryCode } from '../../ui';
import { useLang } from '../../hooks';
import { useGateways, useMainState } from '../../contexts';
import { countriesWithRegions } from '../../constants';
import { QuicTag } from '../node';
import { routes } from '../../router';
import { isBridgeMode, regionToCountryCode, useActionToast } from './util';

type HopSelectProps = {
  node: SelectedNode;
  gatewayId: string | null;
  onClick: () => void;
  nodeHop: NodeHop;
  disabled?: boolean;
};

type SelectedNodeProps = {
  countryCode?: countryCode;
  name: string;
  subInfo?: string | null;
  animate?: boolean;
  quic?: boolean;
  streamOptimized?: boolean;
};

export default function HopSelect({
  nodeHop,
  node,
  gatewayId,
  onClick,
  disabled,
}: HopSelectProps) {
  const { backendFlags, vpnMode, tunnel, connectingState } = useMainState();
  const { lookupGw } = useGateways();
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
    if (disabled) {
      toast();
    } else if (isGateway(node) && gateway) {
      navigate(routes.nodeDetails, {
        state: { gateway, hop: nodeHop, resetScroll: true },
      });
    }
  };

  const nodeData = (selected: SelectedNode, gateway: Gateway | null) => {
    if (selected === 'random') {
      return {
        name: t('fastest', { ns: 'common' }),
        animate: false,
        quic: gateway?.quic,
        streamOptimized: gateway?.asn?.type === 'residential',
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
    return getGatewayInfo(selected.gateway.id, gateway);
  };

  const getLocationInfo = (
    countryCode: string,
    gateway: Gateway | null,
    region?: string,
  ): SelectedNodeProps => {
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
      quic: gateway?.quic,
      streamOptimized: gateway?.asn?.type === 'residential',
    };
  };

  const getGatewayInfo = (
    id: string,
    gateway: Gateway | null,
  ): SelectedNodeProps => {
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
      quic: gateway?.quic,
      streamOptimized: gateway?.asn?.type === 'residential',
    };
  };

  const SelectedNode = ({
    countryCode,
    name,
    subInfo,
    animate,
    quic,
    streamOptimized,
  }: SelectedNodeProps) => {
    const showQuic = quicTag && quic;
    const showStreamOptimized = nodeHop === 'exit' && streamOptimized;
    const showFastest = node === 'random' && !countryCode;

    return (
      <div className="flex flex-row items-center gap-3 overflow-hidden w-full">
        {countryCode && <FlagIcon code={countryCode} alt={countryCode} />}
        {showFastest && (
          <MsIcon
            icon="electric_bolt"
            className="text-2xl text-baltic-sea dark:text-white"
          />
        )}
        <div className={clsx('flex flex-col items-start truncate')}>
          <div
            className={clsx([
              'text-base truncate',
              disabled && 'cursor-default',
            ])}
          >
            {name}
          </div>
          {animate ? (
            <AnimatePresence>
              {subInfo && (
                <motion.div
                  initial={{ opacity: 0, x: '-1rem' }}
                  exit={{ opacity: 0, x: '1rem' }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.2, ease: 'easeOut' }}
                  className="text-sm text-iron dark:text-bombay truncate"
                >
                  {subInfo}
                </motion.div>
              )}
            </AnimatePresence>
          ) : (
            <>
              {subInfo && (
                <div className="text-sm text-iron dark:text-bombay truncate">
                  {subInfo}
                </div>
              )}
            </>
          )}
        </div>
        {(showQuic || showStreamOptimized) && (
          <div className="flex items-center justify-end gap-3 flex-1 mr-1">
            {showStreamOptimized && (
              <MsIcon icon="smart_display" className="text-cornflower" />
            )}
            {showQuic && <QuicTag />}
          </div>
        )}
      </div>
    );
  };

  const gateway = useMemo(() => {
    if (!gatewayId || node === 'random') {
      return null;
    }
    if (isGateway(node)) {
      return lookupGw(node.gateway.id, nodeHop);
    }
    return lookupGw(gatewayId, nodeHop);
  }, [gatewayId, lookupGw, nodeHop, node]);

  return (
    <div
      className={clsx([
        'w-full flex flex-row justify-between items-center h-[3.75rem]',
        'text-baltic-sea dark:text-white',
        'border border-bombay dark:border-iron rounded-lg',
        'relative transition select-none cursor-default',
        disabled && 'opacity-50',
      ])}
      role="presentation"
    >
      <div
        className={clsx([
          'absolute left-3 -top-2.5 px-1',
          'bg-faded-lavender dark:bg-ash text-xs',
          disabled && 'cursor-default',
        ])}
      >
        {nodeHop === 'entry' ? t('first-hop') : t('last-hop')}
      </div>

      <Button
        className={clsx([
          'flex flex-1 pl-4 items-center justify-center h-full py-3 rounded-none rounded-l-lg',
          !disabled && 'hover:text-white/80',
        ])}
        onClick={handleClick}
        onKeyDown={handleClick}
      >
        <SelectedNode {...nodeData(node, gateway)} />
      </Button>
      {isGateway(node) && (
        <Button
          className={clsx(
            'h-11 w-11 my-2 mr-2 flex items-center justify-center rounded-full',
            !disabled && 'hover:bg-mercury dark:hover:bg-mine-shaft',
          )}
          onClick={handleDetailsClick}
          onKeyDown={handleDetailsClick}
        >
          <MsIcon
            icon="arrow_right"
            className="text-baltic-sea dark:text-white leading-none"
          />
        </Button>
      )}
    </div>
  );
}
