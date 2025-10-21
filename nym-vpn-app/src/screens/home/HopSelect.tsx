import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { AnimatePresence, motion } from 'motion/react';
import clsx from 'clsx';
import { Country, Gateway, NodeHop, isGateway } from '../../types';
import { FlagIcon, MsIcon, countryCode } from '../../ui';
import { useLang } from '../../hooks';
import { useGateways, useMainState } from '../../contexts';
import { QuicTag } from '../node';
import { useActionToast } from './util';

type HopSelectProps = {
  node: Country | Gateway;
  gatewayId: string | null;
  onClick: () => void;
  nodeHop: NodeHop;
  disabled?: boolean;
  locked?: boolean;
};

export default function HopSelect({
  nodeHop,
  node,
  gatewayId,
  onClick,
  disabled,
  locked,
}: HopSelectProps) {
  const { backendFlags, quic } = useMainState();
  const { lookupGw } = useGateways();
  const { t } = useTranslation('home');
  const { getCountryName } = useLang();
  const toast = useActionToast('node-select');
  const quicTag =
    nodeHop === 'entry' &&
    backendFlags.quic &&
    quic &&
    (isGateway(node) ? node.quic : true);

  const handleClick = () => {
    if (disabled) {
      toast();
    } else {
      onClick();
    }
  };

  const SelectedCountry = (country: Country, gateway: Gateway | null) => (
    <div
      className="flex flex-row items-center gap-3 overflow-hidden"
      data-testid={`hop-select-country-${nodeHop}`}
    >
      <FlagIcon
        code={country.code.toLowerCase() as countryCode}
        alt={country.code}
        data-testid={`hop-select-flag-${nodeHop}`}
      />
      <div className={clsx('flex flex-col justify-center truncate')}>
        <div
          className={clsx(['text-base truncate', disabled && 'cursor-default'])}
          data-testid={`hop-select-country-name-${nodeHop}`}
        >
          {getCountryName(country.code) || country.name}
        </div>
        <AnimatePresence>
          {gateway && (
            <motion.div
              initial={{ opacity: 0, x: '-1rem' }}
              exit={{ opacity: 0, x: '1rem' }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.2, ease: 'easeOut' }}
              className="text-sm text-iron dark:text-bombay truncate"
            >
              {gateway.name}
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );

  const SelectedGateway = (gateway: Gateway) => (
    <div
      className="flex flex-row items-center gap-3 overflow-hidden"
      data-testid={`hop-select-gateway-${nodeHop}`}
    >
      <FlagIcon
        code={gateway.country.code.toLowerCase() as countryCode}
        alt={gateway.country.code}
        data-testid={`hop-select-gateway-flag-${nodeHop}`}
      />
      <div
        className={clsx(['text-base truncate', disabled && 'cursor-default'])}
        data-testid={`hop-select-gateway-name-${nodeHop}`}
      >
        {gateway.name}
      </div>
    </div>
  );

  const gateway = useMemo(() => {
    if (!gatewayId || isGateway(node)) {
      return null;
    }
    return lookupGw(gatewayId, node.code, nodeHop);
  }, [gatewayId, lookupGw, nodeHop, node]);

  return (
    <div
      className={clsx([
        'w-full flex flex-row justify-between items-center py-3 px-4 h-[3.75rem]',
        'text-baltic-sea dark:text-white',
        'border border-bombay dark:border-iron rounded-lg',
        !locked && [
          'hover:border-baltic-sea hover:ring-baltic-sea',
          'dark:hover:border-white dark:hover:ring-white',
        ],
        'relative transition select-none cursor-default',
        locked && 'opacity-50',
      ])}
      onKeyDown={handleClick}
      role="presentation"
      onClick={handleClick}
      data-testid={`hop-select-${nodeHop}`}
      data-disabled={disabled}
      data-locked={locked}
    >
      <div
        className={clsx([
          'absolute left-3 -top-2.5 px-1',
          'bg-faded-lavender dark:bg-ash text-xs',
          disabled && 'cursor-default',
        ])}
        data-testid={`hop-select-label-${nodeHop}`}
      >
        {nodeHop === 'entry' ? t('first-hop') : t('last-hop')}
      </div>
      {isGateway(node) ? SelectedGateway(node) : SelectedCountry(node, gateway)}
      <div className="flex items-center">
        {quicTag && <QuicTag className="mx-2" />}
        <MsIcon
          icon="arrow_right"
          className="pointer-events-none"
          data-testid={`hop-select-arrow-${nodeHop}`}
        />
      </div>
    </div>
  );
}
