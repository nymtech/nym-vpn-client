import { useCallback, useMemo } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { Button } from '@base-ui/react';
import { useNavigate } from 'react-router';
import { useShallow } from 'zustand/react/shallow';
import {
  FlagIcon,
  LewesIconComponent,
  MsIcon,
  type countryCode,
} from '../../ui';
import { useAppStore, useLookupGw } from '../../store';
import { useLang } from '../../hooks';
import {
  Gateway,
  Score,
  SelectedNode,
  isCountry,
  isGateway,
  isRegion,
} from '../../types';
import { countriesWithRegions } from '../../constants';
import { QuicTag } from '../index';
import { routes } from '../../router';
import { useNodeListState } from '../../store/nodeListState';
import { isBridgeMode, regionToCountryCode } from './util';
import { ScoreIndicatorContainer } from './ScoreIndicatorContainer';

const DURATION = 0.3;

const TEXT_VARIANTS = {
  initial: (k: string) => ({ opacity: 0, x: k === 'name' ? 14 : -14 }),
  animate: { opacity: 1, x: 0 },
  exit: (k: string) => ({ opacity: 0, x: k === 'name' ? -14 : 14 }),
};

type NodeRowProps = {
  type: 'entry' | 'exit';
};

export type SelectedNodeDisplayProps = {
  countryCode?: countryCode;
  name: string;
  location?: string;
  ip?: string;
  showQuic?: boolean;
  disabled?: boolean;
  showStreamOptimized?: boolean;
  showFastest?: boolean;
  score?: Score;
};

export function NodeRow({ type }: NodeRowProps) {
  const { setFocused, addToExpanded, reset } = useNodeListState();

  const { algo, state, userSelectedNode, tunnel, connectingState } =
    useAppStore(
      useShallow((s) => ({
        algo: s.gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm,
        state: s.state,
        userSelectedNode: type === 'entry' ? s.entryNode : s.exitNode,
        tunnel: s.tunnel,
        connectingState: s.connectingState,
      })),
    );

  const navigate = useNavigate();
  const lookupGw = useLookupGw();
  const { getCountryName } = useLang();
  const { t } = useTranslation('home');

  const label = type === 'entry' ? t('nym-entry-server') : t('nym-exit-server');

  const handleClick = () => {
    if (algo === 'auto') return;

    reset(type);

    if (isCountry(userSelectedNode)) {
      setFocused(type, {
        type: 'country',
        key: userSelectedNode.country.code,
      });
    } else if (isRegion(userSelectedNode)) {
      const code = regionToCountryCode(userSelectedNode.region);
      if (code) {
        addToExpanded(type, code.toUpperCase());
        setFocused(type, { type: 'region', key: userSelectedNode.region });
      }
    } else if (isGateway(userSelectedNode)) {
      setFocused(type, { type: 'gateway', key: userSelectedNode.gateway.id });
      const gw = lookupGw(userSelectedNode.gateway.id, type);
      if (gw) {
        addToExpanded(type, gw.country.code.toUpperCase());
        if (gw.country.code.toLowerCase() === 'us') {
          addToExpanded(type, gw.location.region);
        }
      }
      addToExpanded(type, userSelectedNode.gateway.id);
    }

    navigate(routes.nodeLocation, { state: { tab: type } });
  };

  const quicTag =
    type === 'entry' &&
    (isBridgeMode(tunnel?.data) || isBridgeMode(connectingState?.tunnel));

  const gwFlags = useCallback(
    (gw: Gateway | null) => ({
      showQuic: Boolean(quicTag && gw?.quic),
      showStreamOptimized: type === 'exit' && gw?.asn?.type === 'residential',
      showFastest: userSelectedNode === 'random' && !gw?.country?.code,
      score: gw?.type === 'wg' ? gw?.wgScore : gw?.mxScore,
    }),
    [quicTag, type, userSelectedNode],
  );

  const getLocationInfo = useCallback(
    (
      countryCode: string,
      gw: Gateway | null,
      region?: string,
    ): SelectedNodeDisplayProps => {
      const location = getCountryName(countryCode) || countryCode;
      const parts = [location];
      if (region) parts.push(region);
      if (gw) parts.push(gw.location.city);

      return {
        countryCode: countryCode.toLowerCase() as countryCode,
        name: parts.join(', '),
        location: parts.join(', '),
        ip: gw?.exitIpv4 || gw?.exitIpv6 || '',
        ...gwFlags(gw),
      };
    },
    [getCountryName, gwFlags],
  );

  const getGatewayInfo = useCallback(
    (id: string, gw: Gateway | null): SelectedNodeDisplayProps => {
      if (!gw) return { name: id };

      const { country, location, name } = gw;
      const parts: string[] = [];
      if (location.city.length > 0) parts.push(location.city);
      if (
        countriesWithRegions.includes(country.code) &&
        location.region.length > 0
      ) {
        parts.push(location.region);
      }
      parts.push(getCountryName(country.code) || country.name);

      return {
        countryCode: country.code.toLowerCase() as countryCode,
        name,
        location: parts.join(', '),
        ip: gw.exitIpv4 || gw.exitIpv6 || '',
        ...gwFlags(gw),
      };
    },
    [getCountryName, gwFlags],
  );

  const nodeData = useCallback(
    (selected: SelectedNode, gw: Gateway | null): SelectedNodeDisplayProps => {
      if (selected === 'random') {
        return {
          name: t('random', { ns: 'common' }),
          location: 'Random server',
          ip: '',
          ...gwFlags(gw),
        };
      }
      if (isCountry(selected))
        return getLocationInfo(selected.country.code, gw);
      if (isRegion(selected)) {
        return getLocationInfo(
          regionToCountryCode(selected.region) || 'US',
          gw,
          selected.region,
        );
      }
      return getGatewayInfo(selected.gateway.id, gw);
    },
    [getGatewayInfo, getLocationInfo, gwFlags, t],
  );

  const gateway = useMemo(() => {
    const gw =
      type === 'entry'
        ? tunnel?.entryGwId || connectingState?.entryGwId
        : tunnel?.exitGwId || connectingState?.exitGwId;

    switch (algo) {
      case 'auto':
        return gw ? lookupGw(gw, type) : null;
      case 'autoEntryExplicitExit':
      case 'explicit':
        if (isGateway(userSelectedNode))
          return lookupGw(userSelectedNode.gateway.id, type);
        return gw ? lookupGw(gw, type) : null;
    }
  }, [
    algo,
    connectingState?.entryGwId,
    connectingState?.exitGwId,
    lookupGw,
    tunnel?.entryGwId,
    tunnel?.exitGwId,
    type,
    userSelectedNode,
  ]);

  const nodeDetails = useMemo(() => {
    switch (algo) {
      case 'auto':
        return getGatewayInfo(gateway?.id ?? '', gateway);
      case 'autoEntryExplicitExit':
      case 'explicit':
        return nodeData(userSelectedNode, gateway);
    }
  }, [algo, gateway, getGatewayInfo, nodeData, userSelectedNode]);

  const textLabel = useMemo(() => {
    switch (algo) {
      case 'auto':
        return nodeDetails.ip ?? 'Best server for my location';
      case 'autoEntryExplicitExit':
        return state === 'connected' ? nodeDetails.ip : nodeDetails.name;
      case 'explicit':
        return state === 'connected'
          ? (gateway?.name ?? nodeDetails.name)
          : nodeDetails.name;
    }
  }, [algo, gateway, nodeDetails, state]);

  const descriptionLabel = useMemo(() => {
    switch (algo) {
      case 'auto':
        return nodeDetails.location;
      case 'autoEntryExplicitExit':
      case 'explicit':
        return isGateway(userSelectedNode) || state === 'connected'
          ? nodeDetails.location
          : null;
    }
  }, [algo, nodeDetails, state, userSelectedNode]);

  return (
    <>
      <AnimatePresence initial={false}>
        {algo !== 'auto' && (
          <motion.p
            key="label"
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.3, ease: [0.22, 1, 0.36, 1] }}
            className="text-text-secondary overflow-hidden text-xs leading-5 tracking-wide"
          >
            {label}
          </motion.p>
        )}
      </AnimatePresence>
      <Button
        onClick={handleClick}
        className="group relative isolate w-full rounded-xl p-2"
      >
        {/* Rotating gradient ring on hover — mask center with card bg so only border shows */}
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 z-0 rounded-xl opacity-0 transition-opacity duration-200 ease-out group-hover:opacity-100"
        >
          <div className="absolute inset-0 overflow-hidden rounded-[inherit]">
            {/* Outer: translate only. Inner: rotate only — avoids transform override jump on spin */}
            <div className="absolute top-1/2 left-1/2 size-[260%] -translate-x-1/2 -translate-y-1/2">
              <div
                className={clsx(
                  'size-full will-change-transform backface-hidden',
                  '[background:conic-gradient(from_0deg,var(--color-primary)_0%,var(--color-cornflower)_45%,var(--color-azur)_72%,var(--color-primary)_100%)]',
                  'motion-safe:animate-[spin_3s_linear_infinite]',
                )}
              />
            </div>
          </div>
          <div
            className="absolute inset-[2px] rounded-[calc(0.75rem-2px)] bg-white dark:bg-[#1d1d1f]"
            aria-hidden
          />
        </div>

        <div className="relative z-10 flex flex-col items-start">
          <div className="flex w-full items-center justify-between gap-4">
            <div className="flex flex-1 items-center gap-2 overflow-hidden">
              <ScoreIndicatorContainer score={nodeDetails.score} />
              <AnimatePresence mode="wait">
                {nodeDetails.countryCode &&
                  (state === 'connected' || algo !== 'auto') && (
                    <motion.div
                      key={nodeDetails.countryCode}
                      initial={{ opacity: 0, x: 14 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: 14 }}
                      transition={{
                        duration: DURATION,
                        ease: [0.32, 0.72, 0, 1],
                      }}
                    >
                      <FlagIcon
                        code={nodeDetails.countryCode}
                        alt={nodeDetails.name}
                      />
                    </motion.div>
                  )}
              </AnimatePresence>
              <AnimatePresence mode="wait" initial={false}>
                <motion.span
                  key={textLabel}
                  custom={textLabel}
                  variants={TEXT_VARIANTS}
                  initial="initial"
                  animate="animate"
                  exit="exit"
                  transition={{ duration: DURATION, ease: [0.32, 0.72, 0, 1] }}
                  className="text-text-primary block min-w-0 flex-1 truncate overflow-hidden text-start text-base leading-6 tracking-[-0.08px]"
                >
                  {textLabel}
                </motion.span>
              </AnimatePresence>
            </div>
            <div className="flex flex-row items-center justify-center gap-3">
              {nodeDetails.showQuic && <QuicTag />}
              {nodeDetails.showStreamOptimized && (
                <MsIcon icon="smart_display" className="text-cornflower" />
              )}
              {gateway && <LewesIconComponent />}
            </div>
          </div>
          <AnimatePresence initial={false}>
            {descriptionLabel && (
              <motion.div
                key="description"
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: 'auto', opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                transition={{ duration: 0.3, ease: [0.22, 1, 0.36, 1] }}
                className="overflow-hidden"
              >
                <p className="text-text-secondary text-xs leading-5 tracking-[0.18px]">
                  {descriptionLabel}
                </p>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </Button>
    </>
  );
}
