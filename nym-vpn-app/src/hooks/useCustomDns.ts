import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useMainDispatch, useMainState } from '../contexts/main/context';
import { StateDispatch } from '../types';
import { useNewToast } from '../contexts/new-toast-provider/index';

function useCustomDns() {
  const { t } = useTranslation('settings');
  const { customDnsEnabled, customDns, defaultDns } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { add } = useNewToast();

  const toggle = async (state: boolean) => {
    try {
      await invoke('set_custom_dns_enabled', { enabled: state });
      dispatch({ type: 'set-custom-dns-enabled', enabled: state });
    } catch (e) {
      console.error(e);
      add({
        title: t('dns.error.failed'),
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
      add({
        title: t('dns.error.failed'),
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
