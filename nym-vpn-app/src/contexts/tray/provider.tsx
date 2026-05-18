import { useCallback, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useShallow } from 'zustand/react/shallow';
import {
  Gateway,
  SelectedNode,
  isCountry,
  isGateway,
  isRegion,
} from '../../types';
import { useLang } from '../../hooks/index';
import { regionToCountryCode } from '../../screens/home/util';
import { useAppStore, useLookupGw } from '../../store';
import { TrayContext } from './context';

export type TrayProviderProps = {
  children: React.ReactNode;
};
export function TrayProvider({ children }: TrayProviderProps) {
  const { t } = useTranslation('tray');

  const {
    vpnMode,
    state,
    entryNode,
    exitNode,
    tunnel,
    connectingState,
    daemonStatus,
    gatewaySelectionAlgorithm,
  } = useAppStore(
    useShallow((s) => ({
      vpnMode: s.vpnMode,
      state: s.state,
      entryNode: s.entryNode,
      exitNode: s.exitNode,
      tunnel: s.tunnel,
      connectingState: s.connectingState,
      daemonStatus: s.daemonStatus,
      gatewaySelectionAlgorithm:
        s.gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm,
    })),
  );

  const isAutoExit =
    gatewaySelectionAlgorithm === 'auto' ||
    gatewaySelectionAlgorithm === 'autoEntryExplicitExit';

  const lookupGw = useLookupGw();
  const { getCountryName } = useLang();

  const entryGwId = tunnel?.entryGwId || connectingState?.entryGwId || null;
  const exitGwId = tunnel?.exitGwId || connectingState?.exitGwId || null;

  // Vpn Mode
  useEffect(() => {
    let mode = '';
    if (vpnMode === 'mixnet') {
      mode = `${t('mode.mode')}: ${t('mode.anonymous-mixnet')}`;
    } else if (vpnMode === 'wg') {
      mode = `${t('mode.mode')}: ${t('mode.fast-wireguard')}`;
    }
    invoke<void>('update_tray_mode', { mode });
  }, [vpnMode, t]);

  // Connection State
  useEffect(() => {
    let stateValue = '';

    if (daemonStatus === 'auth-denied') {
      stateValue = `${t('state.state')}: ${t('state.auth-denied')}`;
    } else {
      switch (state) {
        case 'connected':
          stateValue = `${t('state.state')}: ${t('state.connected')}`;
          break;
        case 'disconnected':
          stateValue = `${t('state.state')}: ${t('state.disconnected')}`;
          break;
        case 'connecting':
          stateValue = `${t('state.state')}: ${t('state.connecting')}`;
          break;
        case 'disconnecting':
          stateValue = `${t('state.state')}: ${t('state.disconnecting')}`;
          break;
        case 'offline':
          stateValue = `${t('state.state')}: ${t('state.offline')}`;
          break;
        case 'error':
        case 'offline-auto-reconnect':
        case 'unknown':
          stateValue = `${t('state.state')}: ${t('state.error')}`;
          break;
      }
    }
    invoke<void>('update_tray_state', { state: stateValue });
  }, [state, t, daemonStatus]);

  const entryGateway = useMemo(() => {
    if (entryNode === 'random') {
      return null;
    } else if (isGateway(entryNode)) {
      return lookupGw(entryNode.gateway.id, 'entry');
    } else if (entryGwId) {
      return lookupGw(entryGwId, 'entry');
    }
    return null;
  }, [entryNode, lookupGw, entryGwId]);

  const exitGateway = useMemo(() => {
    if (exitNode === 'random') {
      return null;
    } else if (isGateway(exitNode)) {
      return lookupGw(exitNode.gateway.id, 'exit');
    } else if (exitGwId) {
      return lookupGw(exitGwId, 'exit');
    }
    return null;
  }, [exitNode, lookupGw, exitGwId]);

  const getNodeDisplayValue = useCallback(
    (node: SelectedNode, gateway: Gateway | null) => {
      let displayValue: string | null | undefined;
      if (gateway) {
        const location = `${gateway.location.city}, ${getCountryName(gateway.country.code)}`;
        displayValue = `${gateway.name} (${location})`;
      } else if (node === 'random') {
        displayValue = t('random');
      } else if (isCountry(node)) {
        displayValue = getCountryName(node.country.code);
      } else if (isRegion(node)) {
        const country = getCountryName(
          regionToCountryCode(node.region) || 'US',
        );
        displayValue = `${node.region}, ${country}`;
      }
      return displayValue;
    },
    [getCountryName, t],
  );

  // Entry visibility (Explicit shows entry; Auto/AutoEntryExplicitExit hides it)
  useEffect(() => {
    invoke<void>('update_tray_entry_visible', {
      visible: !isAutoExit,
    });
  }, [isAutoExit]);

  // Entry
  useEffect(() => {
    if (isAutoExit) return;
    const displayValue = getNodeDisplayValue(entryNode, entryGateway);
    invoke<void>('update_tray_entry', {
      entry: `${t('entry')}: ${displayValue || '-'}`,
    });
  }, [entryNode, entryGateway, getNodeDisplayValue, t, isAutoExit]);

  // Exit
  useEffect(() => {
    let displayValue: string | null | undefined;
    if (isAutoExit) {
      if (exitGateway) {
        const ip = exitGateway.exitIpv4 ?? exitGateway.exitIpv6;
        const location = `${exitGateway.location.city}, ${getCountryName(exitGateway.country.code)}`;
        displayValue = ip
          ? `${ip} (${location})`
          : `${exitGateway.name} (${location})`;
      } else if (gatewaySelectionAlgorithm === 'auto') {
        displayValue = t('best-server-location');
      } else {
        displayValue = getNodeDisplayValue(exitNode, exitGateway);
      }
    } else {
      displayValue = getNodeDisplayValue(exitNode, exitGateway);
    }
    invoke<void>('update_tray_exit', {
      exit: `${t('exit')}: ${displayValue || '-'}`,
    });
  }, [
    exitNode,
    exitGateway,
    getNodeDisplayValue,
    t,
    gatewaySelectionAlgorithm,
    isAutoExit,
    getCountryName,
  ]);

  // Static tray menu items
  useEffect(() => {
    invoke<void>('update_tray_show_hide', {
      showHide: `${t('show-hide')}`,
    });
    invoke<void>('update_tray_quit', {
      quit: `${t('quit')}`,
    });
  }, [t]);

  return <TrayContext.Provider value={null}>{children}</TrayContext.Provider>;
}
