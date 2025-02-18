import {
  StateDispatch,
  TunnelStateIpc,
  isTunnelConnected,
  isTunnelConnecting,
  isTunnelDisconnecting,
  isTunnelError,
  isTunnelOffline,
} from '../types';

export function tunnelUpdate(state: TunnelStateIpc, dispatch: StateDispatch) {
  if (state === 'disconnected') {
    console.log('tunnel [disconnected]');
    dispatch({ type: 'set-tunnel-disconnected' });
    return;
  }
  if (isTunnelConnected(state)) {
    console.log('tunnel [connected]');
    dispatch({
      type: 'set-tunnel-connected',
      tunnel: state.connected,
    });
    return;
  }
  if (isTunnelConnecting(state)) {
    console.log('tunnel [connecting]');
    dispatch({
      type: 'set-tunnel-connecting',
      tunnel: state.connecting,
    });
    return;
  }
  if (isTunnelDisconnecting(state)) {
    console.log(`tunnel [disconnecting], action ${state.disconnecting}`);
    dispatch({
      type: 'set-tunnel-disconnecting',
      action: state.disconnecting,
    });
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
    dispatch({
      type: 'set-tunnel-inerror',
      error: state.error,
    });
    return;
  }
}
