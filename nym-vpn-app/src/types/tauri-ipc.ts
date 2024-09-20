import { ConnectionState } from './app-state.ts';

export type BackendError = {
  message: string;
  key: BkdErrorKey;
  data: Record<string, string> | null;
};

export type StartupError = { key: StartupErrorKey; details: string | null };

export type Cli = {
  nosplash: boolean;
};

export type DbKey =
  | 'Monitoring'
  | 'Autoconnect'
  | 'UiShowEntrySelect'
  | 'UiTheme'
  | 'UiRootFontSize'
  | 'UiLanguage'
  | 'VpnMode'
  | 'EntryNodeLocation'
  | 'ExitNodeLocation'
  | 'WindowSize'
  | 'WindowPosition'
  | 'WelcomeScreenSeen'
  | 'CredentialExpiry'
  | 'DesktopNotifications';

export type BkdErrorKey =
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
  | 'CredentialInvalid'
  | 'CredentialVpnRunning'
  | 'CredentialAlreadyImported'
  | 'CredentialStorageError'
  | 'CredentialDeserializationFailure'
  | 'CredentialExpired'
  | 'EntryGatewayNotRouting'
  | 'ExitRouterPingIpv4'
  | 'ExitRouterPingIpv6'
  | 'ExitRouterNotRoutingIpv4'
  | 'ExitRouterNotRoutingIpv6'
  | 'UserNoBandwidth'
  | 'WgTunnelError'
  | 'GetMixnetEntryCountriesQuery'
  | 'GetMixnetExitCountriesQuery'
  | 'GetWgCountriesQuery';

export type StartupErrorKey = 'StartupOpenDb' | 'StartupOpenDbLocked';

export type ConnectionStateResponse = {
  state: ConnectionState;
  error?: BackendError | null;
};

export type DaemonInfo = { version: string; network: string };
