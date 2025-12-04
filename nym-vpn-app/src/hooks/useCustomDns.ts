import { invoke } from '@tauri-apps/api/core';
import { useMainDispatch, useMainState } from '../contexts/main/context';
import { StateDispatch } from '../types';
import { useInAppNotify } from '../contexts/index';

function useCustomDns() {
  const { customDnsEnabled, customDns, defaultDns } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { push } = useInAppNotify();

  const toggle = async (state: boolean) => {
    try {
      await invoke('set_custom_dns_enabled', { enabled: state });
      dispatch({ type: 'set-custom-dns-enabled', enabled: state });
    } catch (e) {
      console.error(e);
      push({
        message: 'Failed to apply DNS changes',
        close: true,
        type: 'error',
      });
    }
  };

  const setCustomDns = async (dns: string[]) => {
    try {
      await invoke('set_custom_dns', { dns: dns });
      dispatch({ type: 'set-custom-dns', dns: dns });
    } catch (e) {
      console.error(e);
      push({
        message: 'Failed to apply DNS changes',
        close: true,
        type: 'error',
      });
    }
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
