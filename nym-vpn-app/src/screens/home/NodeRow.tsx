import { useCallback, useMemo } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { useShallow } from 'zustand/react/shallow';
import {
  ButtonIconNew,
  FlagIcon,
  MsIcon,
  Skeleton,
  SmileyIcon,
  type countryCode,
} from '../../ui';
import { useAppStore, useLookupGw } from '../../store';
import { useLang } from '../../hooks';
import {
  Gateway,
  Score,
  SelectedNode,
  isAuto,
  isCountry,
  isGateway,
  isRegion,
} from '../../types';
import { countriesWithRegions } from '../../constants';
import { QuicTag } from '../index';
import { routes } from '../../router';
import { useNodeListState } from '../../store/nodeListState';
import { ScoreIndicator } from '../node/ScoreIndicator';
import { isBridgeMode, regionToCountryCode } from './util';

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
  score?: Score;
};

export function NodeRow({ type }: NodeRowProps) {
  const { setFocused, addToExpanded, reset } = useNodeListState();

  const {
    state,
    userSelectedNode,
    tunnel,
    connectingState,
    vpnMode,
    wgLoading,
    mxEntryLoading,
    mxExitLoading,
    wg,
    mxEntry,
    mxExit,
  } = useAppStore(
    useShallow((s) => ({
      state: s.state,
      userSelectedNode: type === 'entry' ? s.entryNode : s.exitNode,
      tunnel: s.tunnel,
      connectingState: s.connectingState,
      vpnMode: s.vpnMode,
      wgLoading: s.wgLoading,
      mxEntryLoading: s.mxEntryLoading,
      mxExitLoading: s.mxExitLoading,
      wg: s.wg,
      mxEntry: s.mxEntry,
      mxExit: s.mxExit,
    })),
  );

  const navigate = useNavigate();
  const lookupGw = useLookupGw();
  const { getCountryName } = useLang();
  const { t } = useTranslation('home');

  const label = type === 'entry' ? t('nym-entry-server') : t('nym-exit-server');

  const handleClick = () => {
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
      score: gw?.type === 'wg' ? gw?.wgScore : gw?.mxScore,
    }),
    [quicTag, type],
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
          location: t('random-server'),
          ip: '',
          ...gwFlags(gw),
        };
      }
      if (isAuto(selected)) {
        return {
          name: t('safest-server-selection'),
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
      if (isGateway(selected)) {
        return getGatewayInfo(selected.gateway.id, gw);
      }
      return {
        name: t('random', { ns: 'common' }),
        location: t('random-server'),
        ip: '',
        ...gwFlags(gw),
      };
    },
    [getGatewayInfo, getLocationInfo, gwFlags, t],
  );

  // `lookupGw` is a stable store function — depending on it alone won't re-run
  // this memo when the underlying wg/mx lists arrive after a mode switch.
  // Include the list refs so the row updates as soon as gateways load.
  const gateway = useMemo(() => {
    const gw =
      type === 'entry'
        ? tunnel?.entryGwId || connectingState?.entryGwId
        : tunnel?.exitGwId || connectingState?.exitGwId;

    if (isGateway(userSelectedNode)) {
      return lookupGw(userSelectedNode.gateway.id, type);
    }
    return gw ? lookupGw(gw, type) : null;
    // lookupGw is a stable store function that reads wg/mxEntry/mxExit via
    // get() — exhaustive-deps can't see that dependency, but we need to
    // re-run the memo when those lists arrive after a mode switch.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    connectingState?.entryGwId,
    connectingState?.exitGwId,
    lookupGw,
    tunnel?.entryGwId,
    tunnel?.exitGwId,
    type,
    userSelectedNode,
    wg,
    mxEntry,
    mxExit,
  ]);

  const nodeDetails = useMemo(() => {
    // 'random' and 'safest' (the daemon's `Auto`) are both "let the daemon pick
    // each time" selections; once we're connecting/connected we know which
    // gateway it picked — show that instead of the generic placeholder.
    if (
      (userSelectedNode === 'random' || isAuto(userSelectedNode)) &&
      (state === 'connecting' || state === 'connected') &&
      gateway
    ) {
      return getGatewayInfo(gateway.id, gateway);
    }
    return nodeData(userSelectedNode, gateway);
  }, [gateway, getGatewayInfo, nodeData, state, userSelectedNode]);

  // After switching vpnMode the new gateway list may not be loaded yet —
  // lookupGw returns null and we'd otherwise render raw IDs. Show a Loading
  // placeholder until the relevant list (wg / mx-entry / mx-exit) is ready.
  const listLoading =
    (vpnMode === 'wg' && wgLoading) ||
    (vpnMode === 'mixnet' && type === 'entry' && mxEntryLoading) ||
    (vpnMode === 'mixnet' && type === 'exit' && mxExitLoading);

  const hasGatewayIdToResolve =
    Boolean(
      type === 'entry'
        ? tunnel?.entryGwId || connectingState?.entryGwId
        : tunnel?.exitGwId || connectingState?.exitGwId,
    ) || isGateway(userSelectedNode);

  const showLoading = listLoading && !gateway && hasGatewayIdToResolve;

  const textLabel = useMemo(() => {
    return state === 'connected'
      ? (gateway?.name ?? nodeDetails.name)
      : nodeDetails.name;
  }, [gateway?.name, nodeDetails.name, state]);

  // A 'safest' hop carries no gateway id of its own, so `gateway` is null until
  // the daemon reports the one it picked. Gate on that rather than on the
  // tunnel state: at the moment the state flips to 'connecting' the gateway is
  // still unknown, and ScoreIndicator maps an undefined score to a
  // full-strength bar — i.e. a confident signal reading for no server.
  const showSafestPlaceholder = isAuto(userSelectedNode) && !gateway;

  const descriptionLabel = useMemo(() => {
    if (showLoading) return null;
    return isGateway(userSelectedNode) || state === 'connected'
      ? nodeDetails.location
      : null;
  }, [nodeDetails, showLoading, state, userSelectedNode]);

  return (
    <div>
      <p className="text-text-secondary text-xs leading-5 tracking-wide">
        {label}
      </p>
      <div
        role="button"
        tabIndex={0}
        onClick={handleClick}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            handleClick();
          }
        }}
        className="group relative isolate w-full rounded-xl border border-transparent p-2 transition-all duration-150 ease-out hover:border-black dark:hover:border-white"
      >
        <div className="z-10 flex flex-col items-start">
          <div className="flex w-full items-center justify-between gap-4">
            <div className="flex flex-1 items-center gap-2 overflow-hidden">
              {showSafestPlaceholder ? (
                <SmileyIcon className="h-6 w-6" />
              ) : (
                <ScoreIndicator score={nodeDetails.score} />
              )}
              <AnimatePresence mode="wait">
                {nodeDetails.countryCode && (
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
              {showLoading ? (
                <Skeleton className="h-5 w-40" />
              ) : (
                <AnimatePresence mode="wait" initial={false}>
                  <motion.span
                    key={textLabel}
                    custom={textLabel}
                    variants={TEXT_VARIANTS}
                    initial="initial"
                    animate="animate"
                    exit="exit"
                    transition={{
                      duration: DURATION,
                      ease: [0.32, 0.72, 0, 1],
                    }}
                    className="text-text-primary flex min-w-0 flex-1 gap-2 truncate overflow-hidden text-start text-base leading-6 tracking-[-0.08px]"
                  >
                    {textLabel}
                  </motion.span>
                </AnimatePresence>
              )}
            </div>
            <div className="flex flex-row items-center justify-center gap-3">
              {nodeDetails.showQuic && (
                <div>
                  <QuicTag />
                </div>
              )}
              {nodeDetails.showStreamOptimized && (
                <MsIcon icon="smart_display" className="text-status-info" />
              )}
              {gateway && (
                <ButtonIconNew
                  size="small"
                  icon="info"
                  onClick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    navigate(routes.nodeDetails, {
                      state: { gateway, hop: type, resetScroll: true },
                    });
                  }}
                />
              )}
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
      </div>
    </div>
  );
}
