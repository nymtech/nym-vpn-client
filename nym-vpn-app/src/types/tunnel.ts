import { BackendError, ErrorKey } from './tauri-ipc';

export type TunnelConnected = { connected: Tunnel };
export type TunnelConnecting = {
  connecting: Tunnel | null;
};
export type TunnelDisconnecting = { disconnecting: TunnelAction | null };
export type TunnelStateError = { error: TunnelError };
export type TunnelOffline = {
  offline: { reconnect: boolean };
};
type TunnelState =
  | 'disconnected'
  | TunnelConnected
  | TunnelConnecting
  | TunnelDisconnecting
  | TunnelStateError
  | TunnelOffline;
export type TunnelStateIpc = TunnelState;

export function isTunnelConnected(
  state: TunnelState,
): state is TunnelConnected {
  return (state as TunnelConnected).connected !== undefined;
}

export function isTunnelConnecting(
  state: TunnelState,
): state is TunnelConnecting {
  return (state as TunnelConnecting).connecting !== undefined;
}

export function isTunnelDisconnecting(
  state: TunnelState,
): state is TunnelDisconnecting {
  return (state as TunnelDisconnecting).disconnecting !== undefined;
}

export function isTunnelOffline(state: TunnelState): state is TunnelOffline {
  return (state as TunnelOffline).offline !== undefined;
}

export function isTunnelError(state: TunnelState): state is TunnelStateError {
  return (state as TunnelStateError).error !== undefined;
}

export type Tunnel = {
  entryGwId: string;
  exitGwId: string;
  connectedAt: number | null;
  data: TunnelData;
};

export type TunnelData = MixnetData | WireguardData;

export function isMixnetData(data: TunnelData): data is MixnetData {
  return (data as MixnetData).nymAddress !== undefined;
}

export function isWireguardData(data: TunnelData): data is WireguardData {
  return (
    (data as WireguardData).entry !== undefined &&
    (data as WireguardData).exit !== undefined
  );
}

export type TunnelError =
  | 'internal'
  | 'firewall'
  | 'routing'
  | 'dns'
  | 'tun-device'
  | 'tunnel-provider'
  | 'same-entry-and-exit-gw'
  | 'invalid-entry-gw-country'
  | 'invalid-exit-gw-country'
  | 'bad-bandwidth-increase'
  | 'duplicate-tun-fd';

export type TunnelStateEvent = {
  state: TunnelState;
  error: BackendError | null;
};

export type TunnelAction = 'error' | 'reconnect' | 'offline';

export type MixnetData = {
  nymAddress: string | null;
  exitIpr: string | null;
  ipv4: string;
  ipv6: string;
};

export type WireguardData = { entry: WgNode; exit: WgNode };

export type WgNode = {
  endpoint: string;
  publicKey: string;
  privateIpv4: string;
  privateIpv6: string;
};

export type RemainingBandwidth = {
  'remaining-bandwidth': bigint;
};
export type MixnetEvent =
  | 'entry-gw-down'
  | 'exit-gw-down-ipv4'
  | 'exit-gw-down-ipv6'
  | 'exit-gw-routing-error-ipv4'
  | 'exit-gw-routing-error-ipv6'
  | 'connected-ipv4'
  | 'connected-ipv6'
  | 'no-bandwidth'
  | RemainingBandwidth
  | 'sphinx-packet-metrics';

export function isRemainingBandwidth(
  event: MixnetEvent,
): event is RemainingBandwidth {
  return (event as RemainingBandwidth)['remaining-bandwidth'] !== undefined;
}

export type MixnetEventPayload =
  | { event: MixnetEvent }
  | {
      error: ErrorKey;
    };

export function isMixnetEventError(
  payload: MixnetEventPayload,
): payload is { error: ErrorKey } {
  return (payload as { error: ErrorKey }).error !== undefined;
}
