import { ConnectionState } from './app-state';

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
  | 'CSDaemonInternal'
  | 'CSUnhandledExit'
  | 'CStateNoValidCredential'
  | 'CStateTimeout'
  | 'CStateMixnetTimeout'
  | 'CStateMixnetStoragePaths'
  | 'CStateMixnetDefaultStorage'
  | 'CStateMixnetBuildClient'
  | 'CStateMixnetConnect'
  | 'CStateMixnetEntryGateway'
  | 'CStateIprFailedToConnect'
  | 'CStateGwDir'
  | 'CStateGwDirLookupGateways'
  | 'CStateGwDirLookupGatewayId'
  | 'CStateGwDirLookupRouterAddr'
  | 'CStateGwDirLookupIp'
  | 'CStateGwDirEntry'
  | 'CStateGwDirEntryId'
  | 'CStateGwDirEntryLocation'
  | 'CStateGwDirExit'
  | 'CStateGwDirExitLocation'
  | 'CStateGwDirSameEntryAndExitGw'
  | 'CStateOutOfBandwidth'
  | 'CStateOutOfBandwidthSettingUpTunnel'
  | 'CStateBringInterfaceUp'
  | 'CStateFirewallInit'
  | 'CStateFirewallResetPolicy'
  | 'CStateDnsInit'
  | 'CStateDnsSet'
  | 'CStateFindDefaultInterface'
  | 'CSAuthenticatorFailedToConnect'
  | 'CSAuthenticatorConnectTimeout'
  | 'CSAuthenticatorInvalidResponse'
  | 'CSAuthenticatorRegistrationDataVerification'
  | 'CSAuthenticatorEntryGatewaySocketAddr'
  | 'CSAuthenticatorEntryGatewayIpv4'
  | 'CSAuthenticatorWrongVersion'
  | 'CSAuthenticatorMalformedReply'
  | 'CSAuthenticatorAddressNotFound'
  | 'CSAuthenticatorAuthenticationNotPossible'
  | 'CSAddIpv6Route'
  | 'CSTun'
  | 'CSRouting'
  | 'CSWireguardConfig'
  | 'CSMixnetConnectionMonitor'
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
  | 'EntryGatewayNotRouting'
  | 'ExitRouterPingIpv4'
  | 'ExitRouterPingIpv6'
  | 'ExitRouterNotRoutingIpv4'
  | 'ExitRouterNotRoutingIpv6'
  | 'UserNoBandwidth'
  | 'WgTunnelError'
  | 'GetMixnetEntryCountriesQuery'
  | 'GetMixnetExitCountriesQuery'
  | 'GetWgCountriesQuery'
  | 'InvalidNetworkName'
  | 'MaxRegisteredDevices';

export type StartupErrorKey = 'StartupOpenDb' | 'StartupOpenDbLocked';

export type ConnectionStateResponse = {
  state: ConnectionState;
  error?: BackendError | null;
};

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
