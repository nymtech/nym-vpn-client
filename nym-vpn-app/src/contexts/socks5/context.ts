import { createContext } from 'react';
import type {
  HttpRpcSettings,
  SelectedNode,
  Socks5Settings,
  Socks5Status,
} from '../../types';

export type Socks5ContextType = {
  status: Socks5Status | null;
  isLoading: boolean;
  enable: (
    socks5Settings: Socks5Settings,
    httpRpcSettings: HttpRpcSettings,
    exit: SelectedNode,
  ) => Promise<void>;
  disable: () => Promise<void>;
  refresh: () => Promise<void>;
};

const initialState: Socks5ContextType = {
  status: null,
  isLoading: false,
  enable: async () => Promise.resolve(),
  disable: async () => Promise.resolve(),
  refresh: async () => Promise.resolve(),
};

export const Socks5Context = createContext<Socks5ContextType>(initialState);
