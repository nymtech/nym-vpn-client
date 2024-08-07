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
  | 'GetEntryCountriesRequest'
  | 'GetExitCountriesRequest';
