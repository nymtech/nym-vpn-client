import { motion } from 'motion/react';
import clsx from 'clsx';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useShallow } from 'zustand/react/shallow';
import {
  GatewayAnonymousIcon,
  GatewayFastIcon,
} from '../../assets/icons/gateway-mode';
import {
  dispatch,
  useAppStore,
  useFetchGateways,
  useFetchRecents,
} from '../../store';
import { useToast } from '../../hooks';
import { VpnMode } from '../../types';

const MODES = [
  { id: 'fast', Icon: GatewayFastIcon },
  { id: 'mixnet', Icon: GatewayAnonymousIcon },
] as const;

type Mode = (typeof MODES)[number]['id'];

export const ModeToggle = () => {
  const { t } = useTranslation('home');
  const { add } = useToast();
  const fetchGateways = useFetchGateways();
  const fetchRecents = useFetchRecents();

  const { vpnMode } = useAppStore(
    useShallow((s) => ({
      vpnMode: s.vpnMode,
    })),
  );

  const selected: Mode = vpnMode === 'wg' ? 'fast' : 'mixnet';

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
      fetchRecents(mode);
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
    const vpnModeToSet = mode === 'fast' ? 'wg' : 'mixnet';
    await applyVpnMode(vpnModeToSet);
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
