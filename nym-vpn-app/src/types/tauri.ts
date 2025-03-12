import { Country } from './common';

export type BackendError = {
  message: string;
  key: ErrorKey;
  data: Record<string, string> | null;
};

export type StartupError = { key: StartupErrorKey; details: string | null };

export type Cli = {
  nosplash: boolean;
};

export type NetworkEnv = 'mainnet' | 'canary' | 'qa' | 'sandbox';

export type DbKey =
  | 'monitoring'
  | 'ui-theme'
  | 'ui-root-font-size'
  | 'ui-language'
  | 'vpn-mode'
  | 'entry-node'
  | 'exit-node'
  | 'welcome-screen-seen'
  | 'desktop-notifications'
  | 'last-network-env'
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
  | 'unknown-error'
  | 'internal-error'
  | 'grpc-error'
  | 'not-connected-to-daemon'
  | 'entry-gw-down'
  | 'exit-gw-down-ipv4'
  | 'exit-gw-down-ipv6'
  | 'exit-gw-routing-error-ipv4'
  | 'exit-gw-routing-error-ipv6'
  | 'no-bandwidth'
  | 'account-invalid-mnemonic'
  | 'account-storage'
  | 'account-is-connected'
  | 'get-mixnet-entry-countries-query'
  | 'get-mixnet-exit-countries-query'
  | 'get-wg-countries-query'
  | 'invalid-network-name';

export type StartupErrorKey = 'StartupOpenDb' | 'StartupOpenDbLocked';

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

export type Score = 'none' | 'low' | 'medium' | 'high';

export type Gateway = {
  id: string;
  type: GatewayType;
  name: string;
  country: Country;
  mxScore: Score;
  wgScore: Score;
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
