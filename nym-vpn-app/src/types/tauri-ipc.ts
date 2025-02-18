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
  | 'Monitoring'
  | 'Autoconnect'
  | 'UiTheme'
  | 'UiRootFontSize'
  | 'UiLanguage'
  | 'VpnMode'
  | 'EntryNodeLocation'
  | 'ExitNodeLocation'
  | 'WindowSize'
  | 'WindowPosition'
  | 'WelcomeScreenSeen'
  | 'DesktopNotifications';

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

export type VpndStatus = VpndOk | VpndNonCompat | 'notOk';

export function isVpndOk(status: VpndStatus): status is VpndOk {
  return status !== 'notOk' && (status as VpndOk).ok !== undefined;
}

export function isVpndNonCompat(status: VpndStatus): status is VpndNonCompat {
  return (
    status !== 'notOk' && (status as VpndNonCompat).nonCompat !== undefined
  );
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
