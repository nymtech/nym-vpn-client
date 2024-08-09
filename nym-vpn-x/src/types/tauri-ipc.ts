import { ConnectionState } from './app-state.ts';

export type BackendError = {
  message: string;
  key: BkdErrorKey;
  data: Record<string, string> | null;
};

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
  | 'ConnectionTimeout'
  | 'ConnectionGatewayLookup'
  | 'ConnectionNoValidCredential'
  | 'ConnectionSameEntryAndExitGw'
  | 'CredentialInvalid'
  | 'CredentialVpnRunning'
  | 'CredentialAlreadyImported'
  | 'CredentialStorageError'
  | 'CredentialDeserializationFailure'
  | 'CredentialExpired'
  | 'OutOfBandwidth'
  | 'EntryGatewayNotRouting'
  | 'ExitRouterPingIpv4'
  | 'ExitRouterNotRoutingIpv4'
  | 'UserNoBandwidth'
  | 'GetEntryCountriesQuery'
  | 'GetExitCountriesQuery';

export type ConnectionStateResponse = {
  state: ConnectionState;
  error?: BackendError | null;
};
