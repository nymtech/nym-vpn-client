import { useContext } from 'react';
import { Socks5Context } from './context';

export function useSocks5() {
  const context = useContext(Socks5Context);
  if (!context) {
    throw new Error('useSocks5 must be used within a Socks5Provider');
  }
  return context;
}

