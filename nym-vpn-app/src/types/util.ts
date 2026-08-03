import {
  ConnectingState,
  Country,
  ErrorKey,
  Gateway,
  MixnetData,
  MixnetEvent,
  MixnetEventPayload,
  Region,
  SelectedNode,
  TAccountState,
  TBackendError,
  TTunnelState,
  Tunnel,
  TunnelAction,
  TunnelData,
  TunnelError,
  VpndInfo,
  VpndStatus,
  WireguardData,
} from './tauri';

export type NodeHop = 'entry' | 'exit';

type ErrorDataKey = keyof Pick<TBackendError, 'data'>;
// Rust `HashMap` is generated with optional key index `[key in string]?:`
// this is quite painful to work with and can be safely overridden to
// required key
// see https://github.com/Aleph-Alpha/ts-rs/pull/339
export type BackendError = Omit<TBackendError, 'data'> &
  Record<ErrorDataKey, Record<string, string> | null>;

// Flattened derived state from `TTunnelState`
export type TunnelState =
  | 'connected'
  | 'disconnected'
  | 'connecting'
  | 'disconnecting'
  | 'error'
  | 'offline'
  | 'offline-auto-reconnect'
  // when not connected to the daemon, the state is unknown
  | 'unknown';

export type TunnelConnected = { connected: Tunnel };
export type TunnelConnecting = {
  connecting: ConnectingState;
};
export type TunnelDisconnecting = { disconnecting: TunnelAction | null };
export type TunnelStateError = { error: TunnelError };
export type TunnelOffline = {
  offline: { reconnect: boolean };
};

export function isTunnelConnected(
  state: TTunnelState,
): state is TunnelConnected {
  return (state as TunnelConnected).connected !== undefined;
}

export function isTunnelConnecting(
  state: TTunnelState,
): state is TunnelConnecting {
  return (state as TunnelConnecting).connecting !== undefined;
}

export function isTunnelDisconnecting(
  state: TTunnelState,
): state is TunnelDisconnecting {
  return (state as TunnelDisconnecting).disconnecting !== undefined;
}

export function isTunnelOffline(state: TTunnelState): state is TunnelOffline {
  return (state as TunnelOffline).offline !== undefined;
}

export function isTunnelError(state: TTunnelState): state is TunnelStateError {
  return (state as TunnelStateError).error !== undefined;
}

export function isMixnetData(data: TunnelData): data is MixnetData {
  return (data as MixnetData).nymAddress !== undefined;
}

export function isWireguardData(data: TunnelData): data is WireguardData {
  return (
    (data as WireguardData).entry !== undefined &&
    (data as WireguardData).exit !== undefined
  );
}

export type RemainingBandwidth = {
  'remaining-bandwidth': bigint;
};

export function isRemainingBandwidth(
  event: MixnetEvent,
): event is RemainingBandwidth {
  return (event as RemainingBandwidth)['remaining-bandwidth'] !== undefined;
}

export function isMixnetEventError(
  payload: MixnetEventPayload,
): payload is { error: ErrorKey } {
  return (payload as { error: ErrorKey }).error !== undefined;
}

type VpndOk = { ok: VpndInfo | null };
type VpndNonCompat = {
  nonCompat: {
    // The current daemon version and network
    current: VpndInfo;
    // The SemVer version requirement
    requirement: string;
  };
};

export function isVpndOk(status: VpndStatus): status is VpndOk {
  return status !== 'down' && (status as VpndOk).ok !== undefined;
}

export function isVpndNonCompat(status: VpndStatus): status is VpndNonCompat {
  return status !== 'down' && (status as VpndNonCompat).nonCompat !== undefined;
}

export type SelectedGateway = {
  gateway: {
    id: string;
  };
};
export function isGateway(node: SelectedNode): node is SelectedGateway {
  return (node as SelectedGateway).gateway !== undefined;
}
export type SelectedCountry = {
  country: {
    code: string;
  };
};
export function isCountry(node: SelectedNode): node is SelectedCountry {
  return (node as SelectedCountry).country !== undefined;
}
export type SelectedEgion = {
  region: string;
};
export function isRegion(node: SelectedNode): node is SelectedEgion {
  return (node as SelectedEgion).region !== undefined;
}
export type SelectedAuto = {
  auto: {
    exclude_user_country: boolean;
    exclude_entry_point_country: boolean;
  };
};
export function isAuto(node: SelectedNode): node is SelectedAuto {
  return (node as SelectedAuto).auto !== undefined;
}

export type SelectableNode = Gateway | Country | Region | 'random';
function isSelectableGateway(node: SelectableNode): node is Gateway {
  return (
    (node as Gateway).id !== undefined && (node as Gateway).type !== undefined
  );
}
function isSelectableCountry(node: SelectableNode): node is Country {
  return (
    (node as Country).code !== undefined &&
    (node as Country).name !== undefined &&
    (node as Gateway).id === undefined
  );
}
export function toSelectedNode(node: SelectableNode): SelectedNode {
  if (node === 'random') {
    return 'random';
  }
  if (isSelectableGateway(node)) {
    return {
      gateway: { id: node.id },
    };
  } else if (isSelectableCountry(node)) {
    return {
      country: { code: node.code },
    };
  } else {
    return { region: node.name };
  }
}

export type AccountStateError = {
  error: BackendError;
};

// Flattened derived state from `TAccountState`
export type AccountState =
  Exclude<TAccountState, 'syncing' | AccountStateError> | 'error';

export function isAccountError(
  state: TAccountState,
): state is AccountStateError {
  return (state as AccountStateError).error !== undefined;
}

export type CodeDependency = {
  name: string;
  version?: string;
  licenses: string[];
  repository?: string;
  authors: string[];
  copyright?: string;
};

export type DaemonStatus = 'ok' | 'non-compat' | 'down' | 'auth-denied';

export type ProgressMsg = 'canceling';

export type AppError = {
  message: string;
  key: ErrorKey;
  data?: Record<string, string> | null;
};

export function isTunnelInternalError(
  error: TunnelError,
): error is { internal: string } {
  return (error as { internal: string }).internal !== undefined;
}
