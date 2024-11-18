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
  | 'ConnectGeneral'
  | 'ConnectNoAccountStored'
  | 'ConnectNoDeviceStored'
  | 'ConnectUpdateAccount'
  | 'ConnectUpdateDevice'
  | 'ConnectRegisterDevice'
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
  | 'InvalidNetworkName';

export type StartupErrorKey = 'StartupOpenDb' | 'StartupOpenDbLocked';

export type ConnectionStateResponse = {
  state: ConnectionState;
  error?: BackendError | null;
};

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
