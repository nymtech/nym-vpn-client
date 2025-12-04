import { invoke } from '@tauri-apps/api/core';
import { useMainDispatch, useMainState } from '../contexts/main/context';
import { StateDispatch } from '../types';

function useCustomDns() {
  const { customDnsEnabled, customDns, defaultDns } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  const toggle = async (state: boolean) => {
    await invoke('set_custom_dns_enabled', { enabled: state });
    dispatch({ type: 'set-custom-dns-enabled', enabled: state });
  };

  const setCustomDns = async (dns: string[]) => {
    await invoke('set_custom_dns', { dns: dns });
    dispatch({ type: 'set-custom-dns', dns: dns });
  };

  return {
    enabled: customDnsEnabled,
    toggle,
    customDns,
    defaultDns,
    setCustomDns,
  };
}

export default useCustomDns;
