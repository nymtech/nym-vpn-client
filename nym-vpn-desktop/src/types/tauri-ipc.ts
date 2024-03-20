export type CmdErrorSource = 'InternalError' | 'CallerError' | 'Unknown';

export interface CmdError {
  source: CmdErrorSource;
  message: string;
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
  | 'ExitNodeLocation';
