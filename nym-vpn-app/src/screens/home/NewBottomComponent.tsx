import { useEffect, useState } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import clsx from 'clsx';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { ButtonNew, ButtonVariant, MsIcon, type countryCode } from '../../ui';
import {
  dispatch,
  useAppStore,
  useFetchGateways,
  useMainState,
} from '../../store';
import { useToast } from '../../hooks';
import { useAnimatedNavigate } from '../../hooks/useAnimatedNavigate';
import { routes } from '../../router';
import { GatewaySelectionAlgorithm, Score, VpnMode } from '../../types';
import { InteractiveCard } from './InteractiveCard';
import { NodeRow } from './NodeRow';

export type FoldState = 0 | 1 | 2;

const DURATION = 0.3;
const easeOutQuart = [0.22, 1, 0.36, 1] as const;

type ChevronProps = { onUp?: () => void; onDown?: () => void };

function Chevrons({ onUp, onDown }: ChevronProps) {
  if (!onUp && !onDown) return null;

  return (
    <motion.div
      initial={{ x: -24, opacity: 0, width: 0, marginLeft: -8 }}
      animate={{ x: 0, opacity: 1, width: 'auto', marginLeft: 0 }}
      exit={{ x: 24, opacity: 0, width: 0, marginLeft: -8 }}
      transition={{ duration: DURATION, ease: easeOutQuart }}
      style={{ overflow: 'hidden' }}
      className="flex shrink-0 flex-col items-center"
    >
      <button
        type="button"
        onClick={onUp}
        className={clsx([
          'text-text-secondary hover:text-baltic-sea cursor-default leading-none transition-all dark:hover:text-white',
          onUp ? 'opacity-100' : 'opacity-0',
        ])}
      >
        <MsIcon icon="keyboard_arrow_up" className="text-xl! leading-none" />
      </button>
      <button
        type="button"
        onClick={onDown}
        className={clsx([
          'text-text-secondary hover:text-baltic-sea cursor-default leading-none transition-all dark:hover:text-white',
          onDown ? 'opacity-100' : 'opacity-0',
        ])}
      >
        <MsIcon icon="keyboard_arrow_down" className="text-xl! leading-none" />
      </button>
    </motion.div>
  );
}

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

function ModeToggle() {
  const { t } = useTranslation('home');

  const { add } = useToast();
  const vpnMode = useAppStore((s) => s.vpnMode);
  const state = useAppStore((s) => s.state);

  const fetchGateways = useFetchGateways();

  const isFast = vpnMode === 'wg';

  const toggleDisabled = state === 'connected' || state === 'connecting';

  const handleToggle = async (mode: VpnMode) => {
    if (mode === vpnMode || toggleDisabled) return;
    try {
      await invoke('set_vpn_mode', { mode });
      dispatch({ type: 'set-vpn-mode', mode });
      console.info(`vpn mode set to [${mode}]`);
      if (mode === 'mixnet') {
        fetchGateways('mx-entry');
        fetchGateways('mx-exit');
      } else {
        fetchGateways('wg');
      }
    } catch (error: unknown) {
      console.error(`failed to set vpn mode to [${mode}]`, error);
      add({
        id: 'vpn-mode-toggle-error',
        title: t('toggle-vpn-mode.error'),
        type: 'error',
      });
    }
  };

  return (
    <div className="flex items-center justify-between gap-4">
      <div className="flex min-w-0 flex-1 items-center justify-center gap-4">
        <button
          type="button"
          onClick={() => handleToggle('wg')}
          className={clsx(
            'w-20 shrink-0 cursor-default text-right text-sm leading-[22px] tracking-[0.07px] transition-colors',
            isFast
              ? 'text-primary font-bold'
              : 'text-secondary hover:text-baltic-sea dark:hover:text-white',
          )}
        >
          {t('toggle-vpn-mode.fast')}
        </button>

        {/* Toggle pill */}
        <button
          type="button"
          onClick={() => handleToggle(isFast ? 'mixnet' : 'wg')}
          aria-label={t('toggle-vpn-mode.aria-label')}
          className={clsx(
            'dark:bg-background bg-gray relative h-10 w-20 shrink-0 cursor-default rounded-full',
            toggleDisabled && 'cursor-not-allowed opacity-50',
          )}
        >
          <motion.div
            className="dark:bg-charcoal pointer-events-none absolute top-1.5 flex size-7 items-center justify-center rounded-full bg-white"
            animate={{
              x: isFast ? 6 : 40,
            }}
            initial={false}
            transition={{ type: 'spring', stiffness: 420, damping: 32 }}
          >
            <AnimatePresence mode="wait" initial={false}>
              <motion.span
                key={isFast ? 'electric_bolt' : 'visibility_off'}
                initial={{ opacity: 0, rotateX: 90 }}
                animate={{ opacity: 1, rotateX: 0 }}
                exit={{ opacity: 0, rotateX: -90 }}
                transition={{ duration: 0.1 }}
                style={{ fontVariationSettings: "'FILL' 1" }}
                className={clsx([
                  'font-icon inline-block text-2xl select-none rtl:-scale-x-100',
                  'shrink-0 text-xl!',
                  'text-primary',
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
          onClick={() => handleToggle('mixnet')}
          className={clsx(
            'w-20 shrink-0 cursor-default text-sm leading-[22px] tracking-[0.07px] transition-colors',
            !isFast
              ? 'text-primary font-bold'
              : 'text-secondary hover:text-primary',
          )}
        >
          {t('toggle-vpn-mode.anonymous')}
        </button>
      </div>
    </div>
  );
}

export function NewBottomComponent() {
  const navigate = useAnimatedNavigate();
  const { t } = useTranslation('home');
  const { state, daemonStatus, accountState, account } = useMainState();

  const chevronsDisabled =
    state === 'connected' ||
    state === 'connecting' ||
    state === 'offline-auto-reconnect' ||
    state === 'error';

  const daemonUnavailable =
    daemonStatus === 'auth-denied' || daemonStatus === 'down';
  const needAPlan =
    !daemonUnavailable &&
    state === 'disconnected' &&
    account &&
    (accountState === 'no-subscription' ||
      accountState === 'bandwidth-exceeded');

  const { add } = useToast();
  const gatewaySelectionAlgorithmConfig = useAppStore(
    (s) => s.gatewaySelectionAlgorithmConfig,
  );

  const [foldState, setFoldState] = useState<FoldState>(() => {
    switch (gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm) {
      case 'auto':
        return 0;
      case 'autoEntryExplicitExit':
        return 1;
      case 'explicit':
        return 2;
    }
  });

  const expand = () => setFoldState((s) => Math.min(s + 1, 2) as FoldState);
  const collapse = () => setFoldState((s) => Math.max(s - 1, 0) as FoldState);

  // change gateway selection algorithm config based on fold state
  useEffect(() => {
    (async () => {
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
          title: t('gateway-selection-algorithm.error'),
          type: 'error',
        });
      }
    })();
  }, [add, foldState, gatewaySelectionAlgorithmConfig, t]);

  const handleConnect = async () => {
    if (daemonStatus === 'auth-denied') {
      invoke('retry_authentication').catch((e: unknown) => {
        console.error('retry_authentication failed', e);
      });
      return;
    }

    if (state === 'disconnected' && !account) {
      navigate(routes.onboarding);
      return;
    }

    if (needAPlan) {
      navigate(routes.selectPlan);
      return;
    }

    if (
      state === 'connected' ||
      state === 'connecting' ||
      state === 'offline-auto-reconnect' ||
      state === 'error'
    ) {
      console.info('disconnect attempt');
      dispatch({ type: 'disconnect' });
      try {
        await invoke('disconnect');
      } catch (error: unknown) {
        console.error('failed to disconnect', error);
        add({
          id: 'disconnect-error',
          title: t('failed-to-disconnect'),
          type: 'error',
        });
      }
    }
    if (state === 'disconnected') {
      console.info('connect attempt');
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
    if (daemonStatus === 'auth-denied') {
      return t('authenticate');
    }

    if (!account) {
      return t('get-started');
    }

    if (needAPlan) {
      return t('choose-plan');
    }

    switch (state) {
      case 'connected':
        return t('status.connected');
      case 'disconnected':
      case 'unknown':
        return t('status.disconnected');
      case 'connecting':
      case 'offline-auto-reconnect':
        return t('status.connecting');
      case 'disconnecting':
        return t('status.disconnecting');
      case 'offline':
        return t('status.offline');
      case 'error':
        return t('stop', { ns: 'glossary' });
    }
  };

  const getButtonVariant = (): ButtonVariant => {
    if (!account || needAPlan) {
      return 'primary';
    }

    switch (state) {
      case 'disconnected':
      case 'offline':
        return 'primary';
      case 'connected':
      case 'connecting':
        return 'outlined';
      case 'offline-auto-reconnect':
      case 'disconnecting':
      case 'error':
        return 'destructive';
      case 'unknown':
        return 'outlined';
    }
  };

  const getButtonDisabled = () => {
    if (daemonStatus === 'auth-denied') return false;

    return (
      daemonStatus === 'down' ||
      state === 'offline' ||
      state === 'disconnecting' ||
      accountState === 'pending-subscription'
    );
  };

  return (
    <div className="flex flex-col">
      {/* ── Main card ─────────────────────────────────────────────────────── */}

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
              className="z-10 rounded-t-2xl bg-white px-4 dark:bg-[#1d1d1f]"
            >
              <ModeToggle />
              <motion.div
                initial={{ opacity: 0, width: 0 }}
                animate={{ opacity: 1, width: '100%' }}
                exit={{ opacity: 0, width: 0 }}
                transition={{ duration: DURATION, ease: easeOutQuart }}
                className="bg-background-secondary mx-auto my-4 h-px w-full rounded-full"
              />
            </motion.div>
          )}
        </AnimatePresence>
        {/* ── Toggle section ────────────────────────────────────────────────── */}
        <div className="relative z-20 mb-4 flex flex-col bg-white dark:bg-[#1d1d1f]">
          <div className="flex flex-row items-center gap-2">
            <motion.div className="flex w-full min-w-0 flex-col overflow-hidden">
              <div>
                <NodeRow type="exit" />
              </div>
              <AnimatePresence initial={false}>
                {gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm ===
                  'explicit' && (
                  <motion.div
                    key="entry-node"
                    initial={{ opacity: 0, height: 0 }}
                    animate={{ opacity: 1, height: 'auto' }}
                    exit={{ opacity: 0, height: 0 }}
                    transition={{ duration: DURATION, ease: easeOutQuart }}
                    className="overflow-hidden"
                  >
                    <div className="pt-4">
                      <NodeRow type="entry" />
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>
            </motion.div>
            <AnimatePresence initial={false}>
              {!chevronsDisabled && (
                <Chevrons
                  onUp={foldState < 2 ? expand : undefined}
                  onDown={foldState === 0 ? undefined : collapse}
                />
              )}
            </AnimatePresence>
          </div>
        </div>
        {/* ── Main card ─────────────────────────────────────────────────────── */}

        {/* Button ───────────────────────────────────────────────────────── */}
        <div className="z-10">
          <ButtonNew
            // disabled={daemonStatus === 'down'}
            disabled={getButtonDisabled()}
            variant={getButtonVariant()}
            onClick={handleConnect}
          >
            {getButtonText()}
          </ButtonNew>
        </div>
        {/* Button ───────────────────────────────────────────────────────── */}
      </InteractiveCard>
    </div>
  );
}
