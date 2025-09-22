import { Country } from './common';

export type BackendError = {
  message: string;
  key: ErrorKey;
  data: Record<string, string> | null;
};

export type Cli = {
  nosplash: boolean;
};

export type NetworkEnv = 'mainnet' | 'canary' | 'qa' | 'sandbox';

export type DbKey =
  | 'ui-theme'
  | 'ui-root-font-size'
  | 'ui-language'
  | 'vpn-mode'
  | 'entry-node'
  | 'exit-node'
  | 'welcome-screen-seen'
  | 'desktop-notifications'
  | 'last-network-env'
  | 'disable-ipv6'
  | 'network-stats-enabled'
  | 'quic-enabled'
  | 'domain-fronting-enabled'
  | 'cache-mx-entry-gateways'
  | 'cache-mx-exit-gateways'
  | 'cache-wg-gateways'
  | 'cache-account-id'
  | 'cache-device-id';

/*
 * Enum of the possible specialized errors emitted by the daemon or from the
 * backend side
 * */
export type ErrorKey =
  | 'internal'
  | 'grpc'
  | 'not-connected-to-daemon'
  | 'entry-gw-down'
  | 'exit-gw-down-ipv4'
  | 'exit-gw-down-ipv6'
  | 'exit-gw-routing-error-ipv4'
  | 'exit-gw-routing-error-ipv6'
  | 'mixnet-no-bandwidth'
  | 'account-invalid-mnemonic'
  | 'no-account-stored'
  | 'no-device-stored'
  | 'existing-account'
  | 'bandwidth-exceeded'
  | 'account-status-not-active'
  | 'no-subscription'
  | 'max-device-reached'
  | 'device-time-desync'
  | 'get-mixnet-entry-countries-query'
  | 'get-mixnet-exit-countries-query'
  | 'get-wg-countries-query';

type VpndOk = { ok: DaemonInfo | null };
type VpndNonCompat = {
  nonCompat: {
    // The current daemon version and network
    current: DaemonInfo;
    // The SemVer version requirement
    requirement: string;
  };
};

export type VpndStatus = VpndOk | VpndNonCompat | 'down';

export function isVpndOk(status: VpndStatus): status is VpndOk {
  return status !== 'down' && (status as VpndOk).ok !== undefined;
}

export function isVpndNonCompat(status: VpndStatus): status is VpndNonCompat {
  return status !== 'down' && (status as VpndNonCompat).nonCompat !== undefined;
}

export type DaemonInfo = { version: string; network: NetworkEnv };

export type SystemMessage = {
  name: string;
  message: string;
  properties: Partial<Record<string, string>>;
};

export type AccountLinks = {
  signUp?: string | null;
  signIn?: string | null;
  account?: string | null;
};

export type GatewayType = 'mx-entry' | 'mx-exit' | 'wg';

export type Score = 'offline' | 'low' | 'medium' | 'high';

export type AsnType = 'other' | 'residential';
export type Asn = { asn: string; name: string; type: AsnType };
export type Performance = {
  score: Score;
  load: Score;
  lastUpdatedUtc: string;
  // uptime percentage on the last 24 hours
  uptime24h: number;
};
export type Location = {
  latitude: number;
  longitude: number;
  city: string;
  region: string;
};

export type Gateway = {
  id: string;
  type: GatewayType;
  name: string;
  country: Country;
  location: Location;
  asn: Asn | null;
  mxScore: Score;
  wgScore: Score;
  wgPerformance: Performance | null;
  exitIpv4: string | null;
  exitIpv6: string | null;
  buildVersion: string | null;
};

export type GatewaysByCountry = {
  country: Country;
  gateways: Gateway[];
  type: GatewayType;
};

export function isGateway(node: Gateway | Country): node is Gateway {
  return (
    (node as Gateway).id !== undefined && (node as Gateway).type !== undefined
  );
}

export function isCountry(node: Gateway | Country): node is Country {
  return (
    (node as Country).code !== undefined && (node as Country).name !== undefined
  );
}

export type NetworkCompat = { core: boolean | null; tauri: boolean | null };
export type UpdateMetadata = { version: string; currentVersion: string };
export type DownloadUpdateEvent =
  | { event: 'started'; data: { contentLength: bigint } }
  | { event: 'progress'; data: { chunkLength: number } }
  | { event: 'finished' };

export type GpuType = 'nvidia' | 'amd' | 'intel' | 'unknown';
export type DisplayServer = 'x11' | 'wayland' | 'unknown';
export type OsInfo = {
  name: string;
  kernel: string | null;
  arch: string;
  displayServer?: DisplayServer;
  gpu?: GpuType;
};

export type AccountStateError = {
  error: BackendError;
};
export type TAccountState =
  | 'ready'
  | 'logged-out'
  | 'syncing'
  | 'offline'
  | 'bandwidth-exceeded'
  | 'status-not-active'
  | 'no-subscription'
  | 'max-device-reached'
  | AccountStateError;
export type AccountState =
  | Exclude<TAccountState, 'syncing' | AccountStateError>
  | 'error';

export function isAccountError(
  state: TAccountState,
): state is AccountStateError {
  return (state as AccountStateError).error !== undefined;
}

export type FeatureFlags = {
  quic: boolean;
  domainFronting: boolean;
  zknymCredential: boolean;
  gatewayUpdateVersion: string | null;
  flags: Record<string, string>;
};
