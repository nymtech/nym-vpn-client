import { BackendError, ErrorKey } from './tauri';

export type TunnelConnected = { connected: Tunnel };
export type ConnectingProgress =
  | 'resolving-api-addresses'
  | 'awaiting-account-readiness'
  | 'refreshing-gateways'
  | 'selecting-gateways'
  | 'connecting-mixnet-client'
  | 'connecting-tunnel';
export type ConnectingState = {
  tunnelType: 'wg' | 'mixnet';
  progress: ConnectingProgress;
  tunnel: TunnelData | null;
  retryAttempt: number;
  entryGwId: string | null;
  exitGwId: string | null;
};
export type TunnelConnecting = {
  connecting: ConnectingState;
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
  connectedAt: number | null; // unix timestamp
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
  | { key: 'internal'; message: string | null }
  | { key: 'set-firewall-policy'; message: string | null }
  | { key: 'set-dns'; message: string | null }
  | { key: 'set-routing'; message: string | null }
  | { key: 'same-entry-and-exit-gw'; message: string | null }
  | { key: 'invalid-entry-gw-country'; message: string | null }
  | { key: 'invalid-exit-gw-country'; message: string | null }
  | { key: 'max-devices-reached'; message: string | null }
  | { key: 'bandwidth-exceeded'; message: string | null }
  | { key: 'inactive-subscription'; message: string | null }
  | { key: 'device-time-out-of-sync'; message: string | null }
  | { key: 'ipv6-unavailable'; message: string | null }
  | { key: 'tun-device'; message: string | null }
  | { key: 'tunnel-provider'; message: string | null }
  | { key: 'inactive-account'; message: string | null }
  | { key: 'device-logged-out'; message: string | null }
  | { key: 'credential-wasted-on-entry-gateway'; message: string | null }
  | { key: 'credential-wasted-on-exit-gateway'; message: string | null };

export type TunnelStateEvent = {
  state: TunnelState;
  error: BackendError | null;
};

export type TunnelAction = 'error' | 'reconnect' | 'offline';

export type MxAddress = { nymAddress: string; gatewayId: string };

export type MixnetData = {
  nymAddress: MxAddress | null;
  exitIpr: MxAddress | null;
  ipv4: string;
  ipv6: string | null;
  entryIp: string;
  exitIp: string;
};

export type WireguardData = { entry: WgNode; exit: WgNode };

export type WgNode = {
  endpoint: string;
  publicKey: string;
  privateIpv4: string;
  privateIpv6: string | null;
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
