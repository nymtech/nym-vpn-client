import { createContext } from 'react';
import type {
  NodeConnect,
  Socks5Status,
  Socks5Settings,
  HttpRpcSettings,
} from '../../types';

export type Socks5ContextType = {
  status: Socks5Status | null;
  isLoading: boolean;
  enable: (
    socks5Settings: Socks5Settings,
    httpRpcSettings: HttpRpcSettings,
    exit: NodeConnect,
  ) => Promise<void>;
  disable: () => Promise<void>;
  refresh: () => Promise<void>;
};

export const Socks5Context = createContext<Socks5ContextType | null>(null);
