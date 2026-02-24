import { useCallback, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useMainState } from '../main/context';
import { useGateways } from '../index';
import {
  Gateway,
  SelectedNode,
  isCountry,
  isGateway,
  isRegion,
} from '../../types';
import { useLang } from '../../hooks/index';
import { regionToCountryCode } from '../../screens/home/util';
import { TrayContext } from './context';

export type TrayProviderProps = {
  children: React.ReactNode;
};
export function TrayProvider({ children }: TrayProviderProps) {
  const { t } = useTranslation('tray');

  const { vpnMode, state, entryNode, exitNode, tunnel, connectingState } =
    useMainState();
  const { lookupGw } = useGateways();
  const { getCountryName } = useLang();

  const entryGwId = tunnel?.entryGwId || connectingState?.entryGwId || null;
  const exitGwId = tunnel?.exitGwId || connectingState?.exitGwId || null;

  // Vpn Mode
  useEffect(() => {
    switch (vpnMode) {
      case 'mixnet':
        invoke<void>('update_tray_mode', {
          mode: `${t('mode.mode')}: ${t('mode.anonymous-mixnet')}`,
        });
        break;
      case 'wg':
        invoke<void>('update_tray_mode', {
          mode: `${t('mode.mode')}: ${t('mode.fast-wireguard')}`,
        });
        break;
    }
  }, [vpnMode, t]);

  // Connection State
  useEffect(() => {
    switch (state) {
      case 'connected':
        invoke<void>('update_tray_state', {
          state: `${t('state.state')}: ${t('state.connected')}`,
        });
        break;
      case 'disconnected':
        invoke<void>('update_tray_state', {
          state: `${t('state.state')}: ${t('state.disconnected')}`,
        });
        break;
      case 'connecting':
        invoke<void>('update_tray_state', {
          state: `${t('state.state')}: ${t('state.connecting')}`,
        });
        break;
      case 'disconnecting':
        invoke<void>('update_tray_state', {
          state: `${t('state.state')}: ${t('state.disconnecting')}`,
        });
        break;
      case 'error':
        invoke<void>('update_tray_state', {
          state: `${t('state.state')}: ${t('state.error')}`,
        });
        break;
    }
  }, [state, t]);

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
      if (node === 'random') {
        displayValue = t('random');
      } else if (isCountry(node)) {
        displayValue = getCountryName(node.country.code);
      } else if (isRegion(node)) {
        const country = getCountryName(
          regionToCountryCode(node.region) || 'US',
        );
        displayValue = `${node.region}, ${country}`;
      } else if (gateway) {
        const location = `${gateway.location.city}, ${getCountryName(gateway.country.code)}`;
        displayValue = `${gateway.name} (${location})`;
      }
      return displayValue;
    },
    [getCountryName, t],
  );

  // Entry
  useEffect(() => {
    const displayValue = getNodeDisplayValue(entryNode, entryGateway);
    invoke<void>('update_tray_entry', {
      entry: `${t('entry')}: ${displayValue || '-'}`,
    });
  }, [entryNode, entryGateway, getNodeDisplayValue, t]);

  // Exit
  useEffect(() => {
    const displayValue = getNodeDisplayValue(exitNode, exitGateway);
    invoke<void>('update_tray_exit', {
      exit: `${t('exit')}: ${displayValue || '-'}`,
    });
  }, [exitNode, exitGateway, getNodeDisplayValue, t]);

  // Static tray menu items
  useEffect(() => {
    invoke<void>('update_tray_show_hide', {
      show_hide: `${t('show-hide')}`,
    });
    invoke<void>('update_tray_quit', {
      quit: `${t('quit')}`,
    });
  }, [t]);

  return <TrayContext.Provider value={null}>{children}</TrayContext.Provider>;
}
