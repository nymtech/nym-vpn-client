import { useState } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import clsx from 'clsx';
import { dispatch, useAppStore } from '../../../store/index';
import { useI18nError, useToast } from '../../../hooks/index';
import { BackendError, TAccountMode } from '../../../types/index';
import { routes } from '../../../router';
import { CCache } from '../../../cache/index';

import { ButtonNew, TextArea } from '../../../ui';

export function PassphraseEnter() {
  const [phrase, setPhrase] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');

  const { add, close } = useToast();

  const { daemonStatus, state, technicalOptinSeen } = useAppStore(
    useShallow((s) => ({
      daemonStatus: s.daemonStatus,
      state: s.state,
      technicalOptinSeen: s.technicalOptinSeen,
      uiTheme: s.uiTheme,
    })),
  );

  const navigate = useNavigate();
  const { t } = useTranslation('login');
  const { tE } = useI18nError();

  const onChange = (phrase: string) => {
    setPhrase(phrase);
    setError('');
  };

  const refreshAccountMode = async () => {
    const accountMode = await invoke<TAccountMode>('get_account_mode');
    dispatch({ type: 'set-account-mode', mode: accountMode });
  };

  const handleLogin = async () => {
    if (phrase.length === 0 || loading) return;

    if (state !== 'disconnected') {
      add({
        id: 'tunnel-running-login-error',
        title: t('login.can-t-login-while-tunnel-is-running'),
        type: 'error',
      });
      console.warn(`cannot login while tunnel state is ${state}`);
      return;
    }

    setLoading(true);

    try {
      console.info('logging in');
      await invoke<number | null>('add_account', { mnemonic: phrase.trim() });

      dispatch({ type: 'set-account', stored: true });
      await refreshAccountMode();
      await CCache.del('cache-account-id');
      await CCache.del('cache-device-id');
      dispatch({ type: 'reset-error' });

      if (!technicalOptinSeen) {
        navigate(routes.technicalOptin);
      } else {
        navigate(routes.root);
      }

      close('tunnel-running-login-error');
    } catch (e: unknown) {
      console.error('[login] error', e);
      const eT = e as BackendError;
      const error = tE(eT.key);
      setError(error);
      add({
        id: eT.key,
        title: error,
        type: 'error',
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex h-full flex-col items-center justify-between gap-6">
      <div className="flex flex-col items-center gap-2">
        <h1 className="text-text-primary text-2xl font-medium tracking-tight">
          {t('passphrase.title')}
        </h1>
      </div>
      <div className="flex w-full flex-col gap-3">
        <TextArea
          value={phrase}
          onChange={onChange}
          rows={4}
          resize="none"
          spellCheck={false}
          placeholder={t('passphrase.description')}
          className={clsx(
            'sentry-ignore rounded-xl',
            error && 'border-aphrodisiac!',
          )}
        />
        <ButtonNew
          onClick={handleLogin}
          loading={loading}
          disabled={daemonStatus === 'down' || state !== 'disconnected'}
        >
          {t('passphrase.login-button')}
        </ButtonNew>
      </div>
    </div>
  );
}
