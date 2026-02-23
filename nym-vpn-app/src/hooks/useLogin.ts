import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useMainDispatch, useMainState } from '../contexts';
import { routes } from '../router';
import { CCache } from '../cache';
import {
  BackendError,
  doesAccountNeedSubscription,
  StateDispatch,
  TAccountMode,
  TAccountState,
  TAccountSummary,
} from '../types';
import useDeepLink from './useDeepLink';

export default function useLogin() {
  const [deeplinkLoading, setDeeplinkLoading] = useState(false);
  const [passphraseLoading, setPassphraseLoading] = useState(false);

  const [loginSuccess, setLoginSuccess] = useState(false);

  const { i18n } = useTranslation();
  const { welcomeChecked, accountState } = useMainState();
  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;
  const { startListening } = useDeepLink();

  console.log('mainstate', useMainState());

  useEffect(() => {
    (async () => {
      if (
        loginSuccess &&
        accountState &&
        doesAccountNeedSubscription(accountState)
      ) {
        const accountMode = await invoke<TAccountMode>('get_account_mode');
        dispatch({ type: 'set-account-mode', mode: accountMode });

        const accountSummary = await invoke<TAccountSummary>(
          'get_account_summary',
        );
        console.log('account summary', accountSummary);

        setPassphraseLoading(false);
      }
    })();
  }, [accountState, dispatch, loginSuccess]);

  const onLoginSuccess = useCallback(async () => {
    // if (!welcomeChecked) {
    //   navigate(routes.welcome);
    // } else {
    //   navigate(routes.root);
    // }

    dispatch({ type: 'set-account', stored: true });

    setLoginSuccess(true);

    // const accountMode = await invoke<TAccountMode>('get_account_mode');
    // dispatch({ type: 'set-account-mode', mode: accountMode });

    // // Wait for account state to become ready before fetching account summary
    // const maxWaitMs = 30_000;
    // const pollIntervalMs = 200;
    // const start = Date.now();
    // let accountState: TAccountState | undefined;
    // while (Date.now() - start < maxWaitMs) {
    //   accountState = await invoke<TAccountState>('get_account_state');
    //   if (accountState === 'ready') break;
    //   await new Promise((r) => setTimeout(r, pollIntervalMs));
    // }
    // if (accountState !== 'ready') {
    //   console.warn('Account state did not become ready within timeout');
    // }

    // const accountSummary = await invoke<TAccountSummary>('get_account_summary');
    // console.log('account summary', accountSummary);
    // console.log('subscription-valid-until', accountSummary?.['subscription-valid-until']);

    await CCache.del('cache-account-id');
    await CCache.del('cache-device-id');
    dispatch({ type: 'reset-error' });
  }, [dispatch]);

  const deeplinkLogin = async () => {
    setDeeplinkLoading(true);

    try {
      const loginURl = await invoke<string>('get_deep_link', {
        locale: i18n.language,
        kind: 'Privy',
      });
      openUrl(loginURl);

      const deeplinkUrl = await Promise.race([
        startListening(),
        new Promise<never>((_, reject) =>
          setTimeout(() => reject(new Error('Login timeout')), 300000),
        ),
      ]);

      await invoke('store_deeplink_account', {
        callbackUrl: deeplinkUrl,
      });

      await onLoginSuccess();
      return { error: null };
    } catch (error) {
      return { error: error as Error };
    } finally {
      setDeeplinkLoading(false);
    }
  };

  const passphraseLogin = async (passphrase: string) => {
    setPassphraseLoading(true);

    try {
      console.info('logging in with passphrase');
      await invoke<number | null>('add_account', {
        mnemonic: passphrase.trim(),
      });
      await onLoginSuccess();
      return { error: null };
    } catch (error) {
      setPassphraseLoading(false);
      return { error: error as BackendError };
    } finally {
      // setPassphraseLoading(false);
    }
  };

  return { deeplinkLogin, passphraseLogin, deeplinkLoading, passphraseLoading };
}
