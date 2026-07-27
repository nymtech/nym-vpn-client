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
import { SegmentedToggle, SegmentedToggleItem } from '../../ui';

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

  const items: SegmentedToggleItem<Mode>[] = MODES.map((mode) => ({
    id: mode.id,
    label: t(`mode-toggle.${mode.id}`),
    icon: <mode.Icon className="h-4 w-auto" />,
  }));

  return (
    <SegmentedToggle
      items={items}
      value={selected}
      onChange={handleSelect}
      layoutId="mode-toggle-pill"
    />
  );
};
