import { useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { dispatch, useMainState } from '../store';
import { BackendError } from '../types';
import { CCache } from '../cache';
import useI18nError from './useI18nError';
import { useToast } from './index';

function useLogout() {
  const [loading, setLoading] = useState(false);
  const isLoggingOutRef = useRef(false);

  const { state: tunnelState } = useMainState();
  const { t } = useTranslation('notifications');
  const { tE } = useI18nError();
  const { add } = useToast();

  const performLogout = useCallback(async () => {
    try {
      console.info('logging out');
      await invoke('forget_account');
      dispatch({ type: 'set-account', stored: false });
      dispatch({ type: 'set-account-summary', summary: null });
      await CCache.del('cache-account-id');
      await CCache.del('cache-device-id');
      dispatch({ type: 'reset-error' });

      add({ title: t('logout.success'), type: 'success' });
    } catch (e) {
      console.error('[logout] error', e);
      add({
        title: `${t('logout.error')}: ${tE((e as BackendError).key || 'unknown')}`,
        type: 'error',
      });
    } finally {
      setLoading(false);
      isLoggingOutRef.current = false;
    }
  }, [add, t, tE]);

  // Handle logout completion after disconnect
  useEffect(() => {
    if (!isLoggingOutRef.current) return;

    if (tunnelState === 'disconnected') {
      performLogout();
    }
  }, [tunnelState, performLogout]);

  const logout = useCallback(async () => {
    // Prevent multiple simultaneous logout calls
    if (loading || isLoggingOutRef.current) {
      return;
    }

    setLoading(true);
    isLoggingOutRef.current = true;

    // If already disconnected, proceed directly with logout
    if (tunnelState === 'disconnected') {
      await performLogout();
      return;
    }

    // Need to disconnect first
    if (
      tunnelState === 'connected' ||
      tunnelState === 'connecting' ||
      tunnelState === 'offline-auto-reconnect' ||
      tunnelState === 'error'
    ) {
      try {
        dispatch({ type: 'disconnect' });
        await invoke('disconnect');
        // The effect will handle the rest when state becomes 'disconnected'
      } catch (e: unknown) {
        console.error('[logout] disconnect error', e);
        setLoading(false);
        isLoggingOutRef.current = false;
        add({
          title: `${t('logout.error')}: ${tE((e as BackendError).key || 'unknown')}`,
          type: 'error',
        });
      }
    }
  }, [loading, tunnelState, add, t, tE, performLogout]);

  return { logout, loading };
}

export default useLogout;
