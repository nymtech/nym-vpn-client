// SOCKS5 proxy types matching the serialized backend values
export const Socks5State = {
  Disabled: 'disabled',
  Idle: 'idle',
  Connected: 'connected',
  Error: 'error',
  Unknown: 'unknown',
} as const;

export type Socks5State = (typeof Socks5State)[keyof typeof Socks5State];

export interface Socks5Settings {
  listenAddress: string;
}

export interface HttpRpcSettings {
  listenAddress: string;
}

export interface Socks5Status {
  state: Socks5State;
  socks5Settings?: Socks5Settings;
  httpRpcSettings?: HttpRpcSettings;
  errorMessage?: string;
  activeConnections: number;
}

export function getSocks5StateLabel(state: Socks5State): string {
  switch (state) {
    case Socks5State.Disabled:
      return 'Disabled';
    case Socks5State.Idle:
      return 'Idle';
    case Socks5State.Connected:
      return 'Connected';
    case Socks5State.Error:
      return 'Error';
    default:
      return 'Daemon unreachable';
  }
}
