import { useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useInAppNotify, useMainDispatch, useMainState } from '../contexts';
import { BackendError, StateDispatch } from '../types';
import { CCache } from '../cache';
import useI18nError from './useI18nError';

function useLogout() {
  const [loading, setLoading] = useState(false);
  const isLoggingOutRef = useRef(false);

  const { state } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('notifications');
  const { tE } = useI18nError();
  const { push } = useInAppNotify();

  const performLogout = useCallback(async () => {
    try {
      console.info('logging out');
      await invoke('forget_account');
      dispatch({ type: 'set-account', stored: false });
      await CCache.del('cache-account-id');
      await CCache.del('cache-device-id');
      dispatch({ type: 'reset-error' });

      push({
        message: t('logout.success'),
      });
    } catch (e) {
      console.error('[logout] error', e);
      push({
        message: `${t('logout.error')}: ${tE((e as BackendError).key || 'unknown')}`,
      });
    } finally {
      setLoading(false);
      isLoggingOutRef.current = false;
    }
  }, [dispatch, push, t, tE]);

  // Handle logout completion after disconnect
  useEffect(() => {
    if (!isLoggingOutRef.current) return;

    if (state === 'disconnected') {
      performLogout();
    }
  }, [state, performLogout]);

  const logout = useCallback(async () => {
    // Prevent multiple simultaneous logout calls
    if (loading || isLoggingOutRef.current) {
      return;
    }

    setLoading(true);
    isLoggingOutRef.current = true;

    // If already disconnected, proceed directly with logout
    if (state === 'disconnected') {
      await performLogout();
      return;
    }

    // Need to disconnect first
    if (
      state === 'connected' ||
      state === 'connecting' ||
      state === 'offline-auto-reconnect' ||
      state === 'error'
    ) {
      try {
        dispatch({ type: 'disconnect' });
        await invoke('disconnect');
        // The effect will handle the rest when state becomes 'disconnected'
      } catch (e: unknown) {
        console.error('[logout] disconnect error', e);
        setLoading(false);
        isLoggingOutRef.current = false;
        push({
          message: `${t('logout.error')}: ${tE((e as BackendError).key || 'unknown')}`,
        });
      }
    }
  }, [loading, state, dispatch, push, t, tE, performLogout]);

  return { logout, loading };
}

export default useLogout;
