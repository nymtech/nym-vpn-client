import { useRef, useState } from 'react';
import { motion } from 'motion/react';
import clsx from 'clsx';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useShallow } from 'zustand/react/shallow';
import {
  GatewayAnonymousIcon,
  GatewayFastIcon,
  // GatewayModeAutoIcon,
} from '../../assets/icons/gateway-mode';
import { dispatch, useAppStore, useFetchGateways } from '../../store';
import { useToast } from '../../hooks';
import { ConfirmationDialog } from '../../ui';
import { GatewaySelectionAlgorithm, VpnMode } from '../../types';

const MODES = [
  // { id: 'auto', Icon: GatewayModeAutoIcon },
  { id: 'fast', Icon: GatewayFastIcon },
  { id: 'mixnet', Icon: GatewayAnonymousIcon },
] as const;

type Mode = (typeof MODES)[number]['id'];

// VPN states where switching mode would visibly interrupt the user.
// For these states we surface a confirmation dialog before proceeding.
// Other states (disconnected, disconnecting, error, offline, unknown) either
// have nothing active to interrupt or are already in a transition, so we
// switch silently as before.
const INTERRUPTIVE_STATES = new Set<string>([
  'connected',
  'connecting',
  'offline-auto-reconnect',
]);

export const ModeToggle = () => {
  const { t } = useTranslation('home');
  const { add } = useToast();
  const fetchGateways = useFetchGateways();

  const { algo, vpnMode, gatewaySelectionAlgorithmConfig, state /*, exitNode */ } =
    useAppStore(
      useShallow((s) => ({
        algo: s.gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm,
        vpnMode: s.vpnMode,
        gatewaySelectionAlgorithmConfig: s.gatewaySelectionAlgorithmConfig,
        state: s.state,
        // exitNode: s.exitNode,
      })),
    );

  // Pending mode the user has clicked but not yet confirmed. When non-null,
  // the confirmation dialog is open. We keep the requested mode here so the
  // dialog can render the target mode name and the Confirm handler can apply it.
  const [pendingMode, setPendingMode] = useState<Mode | null>(null);
  // Set while the confirm handler is awaiting backend acknowledgment of the
  // mode switch. Drives the loading indicator on the dialog's Confirm button
  // and blocks double-clicks.
  const [confirmInFlight, setConfirmInFlight] = useState(false);
  // Synchronous re-entry guard. `confirmInFlight` is React state, so it only
  // disables the Confirm button on the next render; two very fast clicks can
  // both pass the `confirmInFlight` check before that render lands and fire
  // performModeChange twice. The ref flips synchronously, closing that window.
  const confirmLockRef = useRef(false);

  const selected: Mode =
    // algo === 'auto' || algo === 'autoEntryExplicitExit'
    //   ? 'auto'
    //   :
    vpnMode === 'wg' ? 'fast' : 'mixnet';

  const applyAlgorithm = async (
    algorithm: GatewaySelectionAlgorithm,
  ): Promise<boolean> => {
    if (algorithm === algo) return true;
    try {
      await invoke('set_gateway_selection_algorithm', { algorithm });
      dispatch({
        type: 'set-gateway-selection-algorithm-config',
        config: {
          ...gatewaySelectionAlgorithmConfig,
          gatewaySelectionAlgorithm: algorithm,
        },
      });
      return true;
    } catch (error: unknown) {
      console.error(
        `failed to set gateway selection algorithm to [${algorithm}]`,
        error,
      );
      add({
        id: 'gateway-selection-algorithm-error',
        title: t('gateway-selection-algorithm.error'),
        type: 'error',
      });
      return false;
    }
  };

  const applyVpnMode = async (mode: VpnMode): Promise<boolean> => {
    if (mode === vpnMode) return true;
    try {
      await invoke('set_vpn_mode', { mode });
      dispatch({ type: 'set-vpn-mode', mode });
      if (mode === 'mixnet') {
        fetchGateways('mx-entry');
        fetchGateways('mx-exit');
      } else {
        fetchGateways('wg');
      }
      return true;
    } catch (error: unknown) {
      console.error(`failed to set vpn mode to [${mode}]`, error);
      add({
        id: 'vpn-mode-toggle-error',
        title: t('toggle-vpn-mode.error'),
        type: 'error',
      });
      return false;
    }
  };

  // Actually apply the mode change. Extracted out of handleSelect so the
  // confirmation dialog's "confirm" path can call it directly.
  const performModeChange = async (mode: Mode) => {
    const vpnModeToSet =
      /* mode === 'auto' || */ mode === 'fast' ? 'wg' : 'mixnet';
    // For Auto, preserve any explicit exit pick: if exitNode is a real
    // selection (country/region/gateway), use 'autoEntryExplicitExit' so the
    // exit row keeps showing it. Only fall back to plain 'auto' when there's
    // no user-picked exit ('random' is the "no pick" sentinel).
    const algorithmToSet: GatewaySelectionAlgorithm =
      // mode === 'auto'
      //   ? exitNode === 'random'
      //     ? 'auto'
      //     : 'autoEntryExplicitExit'
      //   :
      'explicit';
    // Apply algorithm first: it's the higher-level intent. If it fails we
    // bail without touching vpnMode so the UI stays in its previous coherent
    // state instead of half-applying the user's intent.
    const previousAlgorithm = algo;
    const algoOk = await applyAlgorithm(algorithmToSet);
    if (!algoOk) return;
    const vpnOk = await applyVpnMode(vpnModeToSet);
    if (!vpnOk && algorithmToSet !== previousAlgorithm) {
      // vpnMode failed after algorithm changed: roll back so the UI doesn't
      // sit in a half-applied state. applyAlgorithm's early-return guard
      // compares against the captured `algo` so we can't reuse it here.
      try {
        await invoke('set_gateway_selection_algorithm', {
          algorithm: previousAlgorithm,
        });
        dispatch({
          type: 'set-gateway-selection-algorithm-config',
          config: {
            ...gatewaySelectionAlgorithmConfig,
            gatewaySelectionAlgorithm: previousAlgorithm,
          },
        });
      } catch (error: unknown) {
        console.error(
          `failed to rollback gateway selection algorithm to [${previousAlgorithm}]`,
          error,
        );
      }
    }
  };

  const handleSelect = async (mode: Mode) => {
    if (mode === selected) return;
    // If the VPN is currently doing something the user can see (connected,
    // connecting, or about to auto-reconnect from offline), confirm before
    // switching, because applying the new mode triggers an immediate
    // disconnect/reconnect. Without the confirmation users have reported
    // being kicked off the tunnel just to look at the other mode tab.
    if (INTERRUPTIVE_STATES.has(state)) {
      setPendingMode(mode);
      return;
    }
    await performModeChange(mode);
  };

  const handleConfirmSwitch = async () => {
    if (!pendingMode || confirmLockRef.current) return;
    const mode = pendingMode;
    confirmLockRef.current = true;
    setConfirmInFlight(true);
    try {
      await performModeChange(mode);
    } finally {
      confirmLockRef.current = false;
      setConfirmInFlight(false);
      setPendingMode(null);
    }
  };

  const handleCancelSwitch = () => {
    if (confirmInFlight) return;
    setPendingMode(null);
  };

  // Choose the title/description copy based on whether the user is mid-connect
  // or fully connected. Distinguishing these keeps the wording honest about
  // what gets interrupted (a live session vs an in-flight handshake).
  const dialogVariant =
    state === 'connecting' ? 'connecting' : 'connected';
  const dialogTitle = pendingMode
    ? t(`mode-toggle.switch-confirm.title-${dialogVariant}`, {
        mode: t(`mode-toggle.${pendingMode}`),
      })
    : '';
  const dialogDescription = t(
    `mode-toggle.switch-confirm.description-${dialogVariant}`,
  );

  return (
    <>
      <div className="bg-surface-bg relative flex items-center gap-2 rounded-full p-0.5">
        {MODES.map((mode) => {
          const isSelected = selected === mode.id;
          return (
            <button
              key={mode.id}
              type="button"
              onClick={() => handleSelect(mode.id)}
              className={clsx(
                'relative flex flex-1 cursor-default items-center justify-center gap-1.5 rounded-full px-4.5 py-2.5 text-sm font-bold transition-colors',
                isSelected
                  ? 'text-primary'
                  : 'text-text-secondary hover:bg-surface-elev',
              )}
            >
              {isSelected && (
                <motion.div
                  layoutId="mode-toggle-pill"
                  className="bg-surface-elev absolute inset-0 rounded-full"
                  transition={{ duration: 0.3, ease: 'easeOut' }}
                />
              )}
              <mode.Icon className="z-10 h-4 w-auto" />
              <span className="z-10">{t(`mode-toggle.${mode.id}`)}</span>
            </button>
          );
        })}
      </div>
      <ConfirmationDialog
        icon="swap_horiz"
        title={dialogTitle}
        description={dialogDescription}
        confirmButtonText={t('mode-toggle.switch-confirm.confirm')}
        cancelButtonText={t('mode-toggle.switch-confirm.cancel')}
        isOpen={pendingMode !== null}
        isLoading={confirmInFlight}
        onConfirm={handleConfirmSwitch}
        onCancel={handleCancelSwitch}
      />
    </>
  );
};
