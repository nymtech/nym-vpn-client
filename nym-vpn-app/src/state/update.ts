import {
  TAccountState,
  TTunnelState,
  isAccountError,
  isTunnelConnected,
  isTunnelConnecting,
  isTunnelDisconnecting,
  isTunnelError,
  isTunnelOffline,
} from '../types';
import { dispatch } from '../store';

export function updateTunnel(state: TTunnelState) {
  if (state === 'disconnected') {
    console.log('tunnel [disconnected]');
    dispatch({ type: 'set-tunnel-disconnected' });
    return;
  }
  if (isTunnelConnected(state)) {
    console.log('tunnel [connected]');
    dispatch({ type: 'set-tunnel-connected', tunnel: state.connected });
    return;
  }
  if (isTunnelConnecting(state)) {
    console.log(`tunnel [connecting] ${state.connecting.progress}`);
    dispatch({ type: 'set-tunnel-connecting', state: state.connecting });
    return;
  }
  if (isTunnelDisconnecting(state)) {
    console.log(`tunnel [disconnecting], action ${state.disconnecting}`);
    dispatch({ type: 'set-tunnel-disconnecting', action: state.disconnecting });
    return;
  }
  if (isTunnelOffline(state)) {
    console.log(`tunnel [offline], reconnect: ${state.offline.reconnect}`);
    dispatch({
      type: 'set-tunnel-offline',
      reconnect: state.offline.reconnect,
    });
    return;
  }
  if (isTunnelError(state)) {
    console.log('tunnel [error]', state.error);
    if (state.error === 'inactive-subscription') {
      dispatch({ type: 'set-account-state', state: 'no-subscription' });
    }
    dispatch({ type: 'set-tunnel-inerror', error: state.error });
    return;
  }
}

export function updateAccountState(state: TAccountState) {
  console.log(`account state update: ${JSON.stringify(state)}`);
  if (state === 'syncing') {
    dispatch({ type: 'set-account-syncing', syncing: true });
  } else {
    dispatch({ type: 'set-account-syncing', syncing: false });
    if (isAccountError(state)) {
      dispatch({ type: 'set-account-error', error: state.error });
      dispatch({ type: 'set-account-state', state: 'error' });
    } else {
      dispatch({ type: 'set-account-state', state: state });
      dispatch({ type: 'set-account-error', error: null });
    }
  }
}
