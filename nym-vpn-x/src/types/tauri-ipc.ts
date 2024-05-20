export type CmdErrorSource = 'InternalError' | 'CallerError' | 'Unknown';

export interface CmdError {
  source: CmdErrorSource;
  message: string;
  i18n_key?: CmdErrorI18nKey | null;
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

export type CmdErrorI18nKey =
  | 'UnknownError'
  | 'CredentialInvalid'
  | 'CredentialVpnRunning'
  | 'CredentialAlreadyImported'
  | 'CredentialStorageError'
  | 'CredentialDeserializationFailure'
  | 'CredentialExpired';
