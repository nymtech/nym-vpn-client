export interface BackendError {
  message: string;
  key: BkdErrorKey;
  data: Record<string, string> | null;
}

export interface Cli {
  nosplash: boolean;
}

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
  | 'WelcomeScreenSeen';

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
  | 'CredentialExpired';
