import { BackendError } from './tauri-ipc';

export type TunnelConnected = { connected: Tunnel };
export type TunnelConnecting = {
  connecting: Tunnel | null;
};
export type TunnelDisconnecting = { disconnecting: TunnelAction | null };
export type TunnelStateError = { error: TunnelError };
export type TunnelOffline = {
  offline: { reconnect: boolean };
};
export type TunnelState =
  | 'disconnected'
  | TunnelConnected
  | TunnelConnecting
  | TunnelDisconnecting
  | TunnelStateError
  | TunnelOffline;

export function isConnected(state: TunnelState): state is TunnelConnected {
  return (state as TunnelConnected).connected !== undefined;
}

export function isConnecting(state: TunnelState): state is TunnelConnecting {
  return (state as TunnelConnecting).connecting !== undefined;
}

export function isOffline(state: TunnelState): state is TunnelOffline {
  return (state as TunnelOffline).offline !== undefined;
}

export function isError(state: TunnelState): state is TunnelStateError {
  return (state as TunnelStateError).error !== undefined;
}

export type Tunnel = {
  entryGwId: string;
  exitGwId: string;
  connectedAt: bigint | null;
  data: TunnelData;
};

export type TunnelData =
  | { mixnet: MixnetData }
  | {
      wireguard: WireguardData;
    };

export function isMixnetData(data: TunnelData): data is {
  mixnet: MixnetData;
} {
  return (data as { mixnet: MixnetData }).mixnet !== undefined;
}

export function isWireguardData(data: TunnelData): data is {
  wireguard: WireguardData;
} {
  return (data as { wireguard: WireguardData }).wireguard !== undefined;
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
