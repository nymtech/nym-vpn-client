import { useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useGateways, useMainState } from '../index';
import { isGateway } from '../../types/util';

export const useTray = () => {
  const { vpnMode, state, entryNode, tunnel, connectingState } = useMainState();
  const { lookupGw } = useGateways();

  console.log('[useTray] vpnMode', vpnMode);

  useEffect(() => {
    console.log('[useTray] updateing tray mode to', vpnMode);
    switch (vpnMode) {
      case 'mixnet':
        invoke<void>('update_tray_mode', { mode: 'Anonymous(mixnet)' });
        break;
      case 'wg':
        invoke<void>('update_tray_mode', { mode: 'Fast(WireGuard)' });
        break;
    }
  }, [vpnMode]);

  useEffect(() => {
    switch (state) {
      case 'connected':
        invoke<void>('update_tray_state', { state: 'State: Connected' });
        break;
      case 'disconnected':
        invoke<void>('update_tray_state', { state: 'State: Disconnected' });
        break;
      case 'connecting':
        invoke<void>('update_tray_state', { state: 'State: Connecting' });
        break;
      case 'disconnecting':
        invoke<void>('update_tray_state', { state: 'State: Disconnecting' });
        break;
      case 'error':
        invoke<void>('update_tray_state', { state: 'State: Error' });
        break;
    }
  }, [state]);

  const entryGateway = useMemo(() => {
    if (entryNode === 'random') {
      return null;
    }

    console.log('tunnel', tunnel);
    console.log('connectingState', connectingState);
    const entryGwId = tunnel?.entryGwId || connectingState?.entryGwId || null;

    if (isGateway(entryNode)) {
      return lookupGw(entryNode.gateway.id, 'entry');
    } else if (entryGwId) {
      return lookupGw(entryGwId, 'entry');
    }
    return null;
  }, [entryNode, tunnel, connectingState, lookupGw]);

  console.log('entryGateway', entryGateway);
  console.log('entryNode', entryNode);

  useEffect(() => {
    console.log('[useTray] entryGateway', entryGateway);
    // if (entryGateway) {
    //   invoke<void>('update_tray_entry', { entry: `Entry: ${entryGateway}` });
    // }
    // const gateway = isGateway(entryNode) ?
  }, [entryGateway]);
};
