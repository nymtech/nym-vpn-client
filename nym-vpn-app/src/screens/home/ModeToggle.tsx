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
import { GatewaySelectionAlgorithm, VpnMode } from '../../types';

const MODES = [
  // { id: 'auto', Icon: GatewayModeAutoIcon },
  { id: 'fast', Icon: GatewayFastIcon },
  { id: 'mixnet', Icon: GatewayAnonymousIcon },
] as const;

type Mode = (typeof MODES)[number]['id'];

export const ModeToggle = () => {
  const { t } = useTranslation('home');
  const { add } = useToast();
  const fetchGateways = useFetchGateways();

  const { algo, vpnMode, gatewaySelectionAlgorithmConfig /*, exitNode */ } =
    useAppStore(
      useShallow((s) => ({
        algo: s.gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm,
        vpnMode: s.vpnMode,
        gatewaySelectionAlgorithmConfig: s.gatewaySelectionAlgorithmConfig,
        // exitNode: s.exitNode,
      })),
    );

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

  const handleSelect = async (mode: Mode) => {
    if (mode === selected) return;
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

  return (
    <div className="bg-surface-bg relative flex items-center gap-2 rounded-full p-0.5">
      {MODES.map((mode) => {
        const isSelected = selected === mode.id;
        return (
          <button
            key={mode.id}
            type="button"
            aria-pressed={isSelected}
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
  );
};
