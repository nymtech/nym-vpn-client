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
  | 'UnknownError'
  | 'InternalError'
  | 'GrpcError'
  | 'NotConnectedToDaemon'
  | 'EntryGwDown'
  | 'ExitGwDownIpv4'
  | 'ExitGwDownIpv6'
  | 'ExitGwRoutingErrorIpv4'
  | 'ExitGwRoutingErrorIpv6'
  | 'NoBandwidth'
  | 'AccountInvalidMnemonic'
  | 'AccountStorage'
  | 'AccountIsConnected'
  | 'ConnectGeneral'
  | 'ConnectNoAccountStored'
  | 'ConnectNoDeviceStored'
  | 'ConnectUpdateAccount'
  | 'ConnectUpdateDevice'
  | 'ConnectRegisterDevice'
  | 'ConnectRequestZkNym'
  | 'GetMixnetEntryCountriesQuery'
  | 'GetMixnetExitCountriesQuery'
  | 'GetWgCountriesQuery'
  | 'InvalidNetworkName'
  | 'MaxRegisteredDevices';

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

export type ReadyToConnect = 'ready' | { not_ready: string };
