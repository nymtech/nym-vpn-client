import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useMainDispatch, useMainState } from '../contexts/main/context';
import { StateDispatch } from '../types';
import { useInAppNotify } from '../contexts';

function useCustomDns() {
  const { t } = useTranslation('settings');
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
        message: t('dns.error.failed'),
        close: true,
        type: 'error',
      });
    }
  };

  const setCustomDns = async (dns: string[]) => {
    try {
      await invoke('set_custom_dns', { dns });
      dispatch({ type: 'set-custom-dns', dns });
    } catch (e) {
      console.error(e);
      push({
        message: t('dns.error.failed'),
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
