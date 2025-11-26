import { useContext } from 'react';
import { Socks5Context } from './context';

export function useSocks5() {
  return useContext(Socks5Context);
}
