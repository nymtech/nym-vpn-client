import { useCallback, useEffect, useMemo, useState } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import clsx from 'clsx';
import { invoke } from '@tauri-apps/api/core';
import { ButtonNew, FlagIcon, MsIcon, type countryCode } from '../../ui';
import { dispatch, useAppStore, useLookupGw, useMainState } from '../../store';
import { useLang, useToast } from '../../hooks';
import {
  Gateway,
  GatewaySelectionAlgorithm,
  isCountry,
  isGateway,
  isRegion,
  Score,
  SelectedNode,
  VpnMode,
} from '../../types/index';
import { useTranslation } from 'react-i18next';
import { isBridgeMode, regionToCountryCode } from './util';
import { countriesWithRegions } from '../../constants';
import { QuicTag } from '../index';
import { ScoreIndicator } from '../node/ScoreIndicator';
import { Button } from '@base-ui/react';
import { useNavigate } from 'react-router';
import { routes } from '../../router';
import { InteractiveCard } from './InteractiveCard';

type FoldState = 0 | 1 | 2;

type NodeData = { code?: countryCode; name: string; location: string };

const DURATION = 0.3;

const EXIT_NODE: NodeData = {
  code: 'hu',
  name: 'hu-freedom-fight-mixnet',
  location: 'Budapest, Hungary',
};
const ENTRY_NODE: NodeData = {
  code: 'pl',
  name: 'pl-bober-bober-nodersowi',
  location: 'Warsaw, Poland',
};
const DEMO_NODE: NodeData = {
  code: 'ch',
  name: '169.128.6.931',
  location: 'Zurich, Switzerland',
};

const INITIAL_NODE: NodeData = {
  name: 'Best server for my location',
  location: 'Searching best location',
};

type ChevronProps = { onUp?: () => void; onDown?: () => void };

function Chevrons({ onUp, onDown }: ChevronProps) {
  const state = useAppStore((s) => s.state);

  const disabled =
    state === 'connected' ||
    state === 'connecting' ||
    state === 'offline-auto-reconnect' ||
    state === 'error';
  // const disabled = false;

  if (!onUp && !onDown) return null;

  return (
    <div className="flex flex-col items-center shrink-0">
      <button
        disabled={disabled}
        type="button"
        onClick={onUp}
        className={clsx([
          'text-secondary transition-all cursor-default leading-none',
          onUp ? 'opacity-100' : 'opacity-0',
          !disabled && 'hover:text-white',
        ])}
      >
        <MsIcon icon="keyboard_arrow_up" className="text-xl! leading-none" />
      </button>
      <button
        disabled={disabled}
        type="button"
        onClick={onDown}
        className={clsx([
          'text-secondary transition-all cursor-default leading-none',
          onDown ? 'opacity-100' : 'opacity-0',
          !disabled && 'hover:text-white',
        ])}
      >
        <MsIcon icon="keyboard_arrow_down" className="text-xl! leading-none" />
      </button>
    </div>
  );
}

type NodeRowProps = {
  // label?: string;
  type: 'entry' | 'exit';
  foldState: FoldState;
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

function NodeRow({ type, foldState }: NodeRowProps) {
  const gatewaySelectionAlgorithmConfig = useAppStore(
    (s) => s.gatewaySelectionAlgorithmConfig,
  );

  const state = useAppStore((s) => s.state);

  const navigate = useNavigate();
  const userSelectedNode = useAppStore((s) =>
    type === 'entry' ? s.entryNode : s.exitNode,
  );
  const tunnel = useAppStore((s) => s.tunnel);
  const connectingState = useAppStore((s) => s.connectingState);
  const wg = useAppStore((s) => s.wg);

  console.log('[NodeRow] type', type);
  console.log('[NodeRow] userSelectedNode', userSelectedNode);
  console.log('[NodeRow] tunnel', tunnel);
  console.log('[NodeRow] connectingState', connectingState);

  const lookupGw = useLookupGw();

  const gateway = useMemo(() => {
    let gw: string | null | undefined = undefined;
    if (type === 'entry') {
      gw = tunnel?.entryGwId || connectingState?.entryGwId;
    } else {
      gw = tunnel?.exitGwId || connectingState?.exitGwId;
    }
    console.log('[memo][NodeRow] gw', gw);

    let result: Gateway | null = null;

    if (isGateway(userSelectedNode)) {
      result = lookupGw(userSelectedNode.gateway.id, type);
    } else if (gw) {
      result = lookupGw(gw, type);
    }
    console.log('[NodeRow] result', result);
    return result;
  }, [
    connectingState?.entryGwId,
    connectingState?.exitGwId,
    lookupGw,
    userSelectedNode,
    tunnel?.entryGwId,
    tunnel?.exitGwId,
    type,
  ]);

  console.log('[NodeRow] global gateway', gateway);

  useEffect(() => {
    console.log('[NodeRow] wg', wg);
    console.log('[NodeRow] gateway', gateway);
    for (const country of wg) {
      // console.log('[NodeRow] country', country);
      const gwsearch = country.gateways.find((cg) => cg.id === gateway?.id);
      if (gwsearch) {
        console.log('[NodeRow] gwsearch', gwsearch);
      }
    }
  }, [wg, gateway]);

  const label = useMemo(
    () => (type === 'entry' ? 'Nym entry node' : 'Nym exit node'),
    [type],
  );

  const { getCountryName } = useLang();
  const { t } = useTranslation('home');

  const quicConnection =
    isBridgeMode(tunnel?.data) || isBridgeMode(connectingState?.tunnel);
  const quicTag = type === 'entry' && quicConnection;

  // const lookupGw = useLookupGw();

  // const gateway = useMemo(() => {
  //   let gw: string | null | undefined = undefined;
  //   if (type === 'entry') {
  //     gw = tunnel?.entryGwId || connectingState?.entryGwId;
  //   } else {
  //     gw = tunnel?.exitGwId || connectingState?.exitGwId;
  //   }
  //   // console.log('[NodeRow] gw', gw);

  //   let result: Gateway | null = null;

  //   if (isGateway(userSelectedNode)) {
  //     result = lookupGw(userSelectedNode.gateway.id, type);
  //   } else if (gw) {
  //     result = lookupGw(gw, type);
  //   }
  //   return result;
  // }, [
  //   connectingState?.entryGwId,
  //   connectingState?.exitGwId,
  //   lookupGw,
  //   userSelectedNode,
  //   tunnel?.entryGwId,
  //   tunnel?.exitGwId,
  //   type,
  // ]);

  const getLocationInfo = useCallback(
    (
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

      console.log('[NodeRow] getLocationInfo', {
        countryCode,
        location,
        subInfo,
        quicTag,
        gateway,
      });

      return {
        countryCode: countryCode.toLowerCase() as countryCode,
        name: gateway?.name || '',
        location,
        ip: gateway?.exitIpv4 || gateway?.exitIpv6 || '',
        showQuic: Boolean(quicTag && gateway?.quic),
        showStreamOptimized:
          type === 'exit' && gateway?.asn?.type === 'residential',
        showFastest: userSelectedNode === 'random' && !gateway?.country?.code,
        score: gateway?.type === 'wg' ? gateway?.wgScore : gateway?.mxScore,
      };
    },
    [getCountryName, userSelectedNode, quicTag, type],
  );

  const getGatewayInfo = useCallback(
    (id: string, gateway: Gateway | null): SelectedNodeDisplayProps => {
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
        location: components.join(', '),
        ip: gateway?.exitIpv4 || gateway?.exitIpv6 || '',
        showQuic: Boolean(quicTag && gateway?.quic),
        showStreamOptimized:
          type === 'exit' && gateway?.asn?.type === 'residential',
        showFastest: userSelectedNode === 'random' && !gateway?.country?.code,
        score: gateway?.type === 'wg' ? gateway?.wgScore : gateway?.mxScore,
      };
    },
    [getCountryName, userSelectedNode, quicTag, type],
  );

  const nodeData = useCallback(
    (
      selected: SelectedNode,
      gateway: Gateway | null,
    ): SelectedNodeDisplayProps => {
      if (selected === 'random') {
        return {
          name: t('random', { ns: 'common' }),
          location: 'Random server',
          ip: '',
          showQuic: Boolean(quicTag && gateway?.quic),
          showStreamOptimized:
            type === 'exit' && gateway?.asn?.type === 'residential',
          showFastest: userSelectedNode === 'random' && !gateway?.country?.code,
          score: gateway?.type === 'wg' ? gateway?.wgScore : gateway?.mxScore,
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
    },
    [getGatewayInfo, getLocationInfo, userSelectedNode, quicTag, t, type],
  );

  // console.log('[NodeRow] gateway', gateway);
  // console.log('[NodeRow] nodeData', nodeData(node, gateway));

  const nodeDetails = useMemo(() => {
    return nodeData(userSelectedNode, gateway);
  }, [nodeData, userSelectedNode, gateway]);

  // console.log('[NodeRow] foldState', foldState);
  // console.log('[NodeRow] nodeDetails', nodeDetails);

  const getTextLabel = () => {
    switch (gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm) {
      case 'auto':
        if (state === 'connected') {
          return nodeDetails.ip;
        } else {
          return 'Best server for my location';
        }
      case 'autoEntryExplicitExit':
        return nodeDetails.ip ?? 'default ip';
      case 'explicit':
        return nodeDetails.name ?? 'default name';
    }
  };

  // console.log('[NodeRow] getTextLabel', getTextLabel());

  const getTextDescriptionLabel = () => {
    switch (gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm) {
      case 'auto':
        if (state === 'connected') {
          return nodeDetails.location;
        }
        return 'Searching for best location';
      case 'autoEntryExplicitExit':
        return 'Nym exit node';
      case 'explicit':
        return 'Nym entry node';
    }
  };

  // console.log('[NodeRow] getTextDescriptionLabel', getTextDescriptionLabel());

  return (
    <>
      {label &&
        gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm !==
          'auto' && (
          <p className="text-secondary text-xs leading-5 tracking-[0.18px]">
            {label}
          </p>
        )}
      <p>noderow type: {type}</p>
      <Button
        onClick={() =>
          navigate(
            type === 'entry'
              ? routes.entryNodeLocation
              : routes.exitNodeLocation,
          )
        }
        className="group relative isolate rounded-xl p-2 w-full"
      >
        {/* Rotating gradient ring on hover — mask center with card bg so only border shows */}
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 z-0 rounded-xl opacity-0 transition-opacity duration-200 ease-out group-hover:opacity-100"
        >
          <div className="absolute inset-0 overflow-hidden rounded-[inherit]">
            {/* Outer: translate only. Inner: rotate only — avoids transform override jump on spin */}
            <div className="absolute left-1/2 top-1/2 size-[260%] -translate-x-1/2 -translate-y-1/2">
              <div
                className={clsx(
                  'size-full will-change-transform backface-hidden',
                  '[background:conic-gradient(from_0deg,var(--color-malachite-200)_0%,var(--color-cornflower)_45%,var(--color-azur)_72%,var(--color-malachite-200)_100%)]',
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

        <div className="relative z-10 flex flex-col  items-start">
          <div className="flex items-center justify-between gap-4 w-full">
            <div className="flex items-center gap-2 flex-1 overflow-hidden">
              {nodeDetails.score && (
                <ScoreIndicator score={nodeDetails.score} />
              )}

              {nodeDetails.countryCode &&
                (state === 'connected' ||
                  gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm !==
                    'auto') && (
                  <FlagIcon
                    code={nodeDetails.countryCode}
                    alt={nodeDetails.name}
                  />
                )}
              <AnimatePresence mode="wait" initial={false}>
                <motion.span
                  key={foldState === 2 ? 'name' : 'ip'}
                  custom={foldState === 2 ? 'name' : 'ip'}
                  variants={{
                    initial: (k: string) => ({
                      opacity: 0,
                      x: k === 'name' ? 14 : -14,
                    }),
                    animate: { opacity: 1, x: 0 },
                    exit: (k: string) => ({
                      opacity: 0,
                      x: k === 'name' ? -14 : 14,
                    }),
                  }}
                  initial="initial"
                  animate="animate"
                  exit="exit"
                  transition={{ duration: DURATION, ease: [0.32, 0.72, 0, 1] }}
                  className="block truncate flex-1 text-start min-w-0 text-baltic-sea dark:text-white text-base leading-6 tracking-[-0.08px] overflow-hidden"
                >
                  {/* {foldState === 2 ? nodeDetails.name : nodeDetails.ip} */}
                  {getTextLabel()}
                </motion.span>
              </AnimatePresence>
            </div>
            <div className="flex flex-row items-center justify-center gap-3">
              {!nodeDetails.showQuic && <QuicTag />}
              {!nodeDetails.showStreamOptimized && (
                <MsIcon icon="smart_display" className="text-cornflower" />
              )}
            </div>
          </div>
          <p className="ml-10 text-secondary text-xs leading-5 tracking-[0.18px]">
            {/* {nodeDetails.location} */}
            {getTextDescriptionLabel()}
            {/* {getTextLabel()} */}
          </p>
        </div>
      </Button>
    </>
  );
}

function ModeToggle() {
  const { add } = useToast();
  const { vpnMode } = useMainState();

  const isFast = vpnMode === 'wg';

  const handleToggle = async (mode: VpnMode) => {
    if (mode === vpnMode) return;
    try {
      await invoke('set_vpn_mode', { mode });
      dispatch({ type: 'set-vpn-mode', mode });
      console.info(`vpn mode set to [${mode}]`);
      // TODO: fetch gateways?
    } catch (error: unknown) {
      console.error(`failed to set vpn mode to [${mode}]`, error);
      add({
        id: 'vpn-mode-toggle-error',
        title: 'Failed to toggle VPN mode',
        type: 'error',
      });
    }
  };

  return (
    <div className="flex items-center justify-between gap-4">
      <div className="flex items-center gap-4 flex-1 min-w-0 justify-center">
        <button
          type="button"
          onClick={() => handleToggle('wg')}
          className={clsx(
            'text-sm leading-[22px] tracking-[0.07px] w-20 text-right shrink-0 cursor-default transition-colors',
            isFast
              ? 'font-bold text-malachite-200'
              : 'text-secondary hover:text-baltic-sea dark:hover:text-white',
          )}
        >
          Fast
        </button>

        {/* Toggle pill */}
        <button
          type="button"
          onClick={() => handleToggle(isFast ? 'mixnet' : 'wg')}
          aria-label="Toggle VPN mode"
          className="relative bg-[#e5e5e5] dark:bg-[#090909] h-10 w-20 rounded-full shrink-0 cursor-default"
        >
          <motion.div
            className="absolute top-[6px] bg-white border border-ash dark:border-transparent dark:bg-charcoal size-7 rounded-full flex items-center justify-center pointer-events-none"
            animate={{
              x: isFast ? 6 : 40,
              // backgroundColor: 'black',
              // backgroundColor: isFast ? '#d8d8d8' : '#182536',
            }}
            initial={false}
            transition={{ type: 'spring', stiffness: 420, damping: 32 }}
            // style={{ left: 6, right: 6 }}
          >
            <AnimatePresence mode="wait" initial={false}>
              <motion.span
                key={isFast ? 'electric_bolt' : 'visibility_off'}
                initial={{ opacity: 0, rotateX: 90 }}
                animate={{ opacity: 1, rotateX: 0 }}
                exit={{ opacity: 0, rotateX: -90 }}
                transition={{ duration: 0.1 }}
                className={clsx([
                  'font-icon text-2xl select-none inline-block rtl:-scale-x-100',
                  'shrink-0 text-xl!',
                  'text-malachite-200',
                  '[text-shadow:1px_1px_10px_#fff,1px_1px_10px_#ccc]',
                ])}
              >
                {isFast ? 'electric_bolt' : 'visibility_off'}
              </motion.span>
            </AnimatePresence>
          </motion.div>
        </button>

        <button
          type="button"
          onClick={() => handleToggle(isFast ? 'mixnet' : 'wg')}
          className={clsx(
            'text-sm leading-[22px] tracking-[0.07px] w-20 shrink-0 cursor-default transition-colors',
            !isFast
              ? 'font-bold text-malachite-200'
              : // ? 'font-bold text-[#a3cdff]'
                'text-secondary hover:text-baltic-sea dark:hover:text-white',
          )}
        >
          Anonymous
        </button>
      </div>
      {/* <Chevrons onUp={onUp} onDown={onDown} /> */}
    </div>
  );
}

const easeOutQuart = [0.22, 1, 0.36, 1] as const;

export function NewBottomComponent() {
  const { state } = useMainState();

  const { add } = useToast();
  const gatewaySelectionAlgorithmConfig = useAppStore(
    (s) => s.gatewaySelectionAlgorithmConfig,
  );
  // console.log(
  //   '[NewBottomComponent] gatewaySelectionAlgorithmConfig',
  //   gatewaySelectionAlgorithmConfig,
  // );
  const [foldState, setFoldState] = useState<FoldState>(() => {
    switch (gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm) {
      case 'auto':
        return 0;
      case 'autoEntryExplicitExit':
        return 1;
      case 'explicit':
        return 2;
    }
    return 0;
  });

  const expand = () => setFoldState((s) => Math.min(s + 1, 2) as FoldState);
  const collapse = () => setFoldState((s) => Math.max(s - 1, 0) as FoldState);

  // change gateway selection algorithm config based on fold state
  useEffect(() => {
    (async () => {
      // debugger;
      let gatewaySelectionAlgorithm: GatewaySelectionAlgorithm | undefined;
      switch (foldState) {
        case 0:
          gatewaySelectionAlgorithm = 'auto';
          break;
        case 1:
          gatewaySelectionAlgorithm = 'autoEntryExplicitExit';
          break;
        case 2:
          gatewaySelectionAlgorithm = 'explicit';
          break;
      }
      if (
        !gatewaySelectionAlgorithm ||
        gatewaySelectionAlgorithm ===
          gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm
      )
        return;
      try {
        await invoke('set_gateway_selection_algorithm', {
          algorithm: gatewaySelectionAlgorithm,
        });
        dispatch({
          type: 'set-gateway-selection-algorithm-config',
          config: {
            ...gatewaySelectionAlgorithmConfig,
            gatewaySelectionAlgorithm,
          },
        });
      } catch (error: unknown) {
        console.error(
          `failed to set gateway selection algorithm to [${gatewaySelectionAlgorithm}]`,
          error,
        );
        add({
          id: 'gateway-selection-algorithm-error',
          title: 'Failed to set gateway selection algorithm',
          type: 'error',
        });
      }
    })();
  }, [add, foldState, gatewaySelectionAlgorithmConfig]);

  const handleConnect = async () => {
    console.log('handleConnect');

    if (
      state === 'connected' ||
      state === 'connecting' ||
      state === 'offline-auto-reconnect' ||
      state === 'error'
    ) {
      console.log('disconnect attempt');
      dispatch({ type: 'disconnect' });
      try {
        await invoke('disconnect');
      } catch (error: unknown) {
        console.error('failed to disconnect', error);
        add({
          id: 'disconnect-error',
          title: 'Failed to disconnect',
          type: 'error',
        });
      }
    }
    if (state === 'disconnected') {
      console.log('connect attempt');
      dispatch({ type: 'reset-error' });
      dispatch({ type: 'connect' });
      try {
        await invoke('connect');
      } catch (error: unknown) {
        console.error('failed to connect', error);
        add({
          id: 'connect-error',
          title: 'Failed to connect',
          type: 'error',
        });
      }
    }
  };

  const getButtonText = () => {
    switch (state) {
      case 'connected':
        return 'Tap to disconnect';
      case 'disconnected':
        return 'Tap to connect';
      case 'connecting':
        return 'Tap to cancel';
      case 'disconnecting':
        return 'Disconnecting...';
      case 'offline':
        return 'Tap to connect';
    }
  };

  return (
    <div className="flex flex-col">
      <p>fold state: {foldState}</p>
      <p>
        gateway selection algorithm:{' '}
        {gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm}
      </p>
      {/* ── Main card ─────────────────────────────────────────────────────── */}
      {/* <div
        className={clsx(
          'bg-white dark:bg-[#1d1d1f] rounded-2xl px-4 py-4 flex flex-col transition-all duration-300',
          foldState > 0 && 'rounded-t-none',
        )}
      > */}
      <InteractiveCard>
        {/* ── Toggle section ────────────────────────────────────────────────── */}
        {/* Slides up from below when entering states 1/2 */}
        <AnimatePresence initial={false}>
          {foldState > 0 && (
            <motion.div
              key="toggle-header"
              initial={{ y: '100%', height: 0 }}
              animate={{ y: 0, height: 'auto' }}
              exit={{ y: '100%', height: 0 }}
              transition={{ duration: DURATION, ease: easeOutQuart }}
              className="z-10 bg-white dark:bg-[#1d1d1f] rounded-t-2xl px-4"
            >
              <ModeToggle />
              <div className="h-px bg-[#3b3b3b] rounded-full w-full my-4" />
            </motion.div>
          )}
        </AnimatePresence>
        {/* ── Toggle section ────────────────────────────────────────────────── */}
        <div className="relative flex flex-col mb-4 z-20 bg-white dark:bg-[#1d1d1f] ">
          <div className="flex flex-row gap-2 items-center">
            <motion.div
              className={clsx(
                'w-full min-w-0 flex flex-col overflow-hidden',
                foldState === 2 && 'gap-4',
              )}
            >
              <div>
                <NodeRow
                  type={
                    gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm ===
                    'explicit'
                      ? 'entry'
                      : 'exit'
                  }
                  // {...ENTRY_NODE}
                  // {...DEMO_NODE}
                  // {...INITIAL_NODE}
                  // label={foldState > 0 ? 'Nym entry node' : undefined}
                  // label="Nym entry node"
                  // onUp={foldState === 0 ? expand : undefined}
                  foldState={foldState}
                />
              </div>
              <AnimatePresence initial={false}>
                {gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm ===
                  'explicit' && (
                  <motion.div
                    key="exit-node"
                    initial={{ opacity: 0, y: '100%', height: 0 }}
                    animate={{ opacity: 1, y: 0, height: 'auto' }}
                    exit={{ opacity: 0, y: '100%', height: 0 }}
                    transition={{ duration: DURATION, ease: easeOutQuart }}
                  >
                    <NodeRow
                      type="exit"
                      // {...EXIT_NODE}
                      // label="Nym exit node"
                      foldState={foldState}
                    />
                  </motion.div>
                )}
              </AnimatePresence>
            </motion.div>
            <Chevrons
              onUp={foldState < 2 ? expand : undefined}
              onDown={foldState === 0 ? undefined : collapse}
            />
          </div>
        </div>
        {/* ── Main card ─────────────────────────────────────────────────────── */}

        {/* Button ───────────────────────────────────────────────────────── */}
        <div className="z-10">
          <ButtonNew variant="outlined" onClick={handleConnect}>
            {getButtonText()}
          </ButtonNew>
        </div>
        {/* Button ───────────────────────────────────────────────────────── */}
      </InteractiveCard>
    </div>
  );
}
