import { Country } from './common';

export type BackendError = {
  message: string;
  key: BkdErrorKey;
  data: Record<string, string> | null;
};

export type Cli = {
  nosplash: boolean;
};

export type NodeLocationBackend = 'Fastest' | { Country: Country };

export type DbKey =
  | 'Monitoring'
  | 'Autoconnect'
  | 'EntryLocationEnabled'
  | 'UiTheme'
  | 'UiRootFontSize'
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
  | 'CredentialInvalid'
  | 'CredentialVpnRunning'
  | 'CredentialAlreadyImported'
  | 'CredentialStorageError'
  | 'CredentialDeserializationFailure'
  | 'CredentialExpired'
  | 'GetEntryCountriesRequest'
  | 'GetExitCountriesRequest';
