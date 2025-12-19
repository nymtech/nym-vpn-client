import { useState } from 'react';
import clsx from 'clsx';
import { motion } from 'motion/react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useNavigate } from 'react-router';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Button, MsIcon, PageAnim, TextArea } from '../../../ui';
import { NymVpnPricingUrl } from '../../../constants';
import {
  useInAppNotify,
  useMainDispatch,
  useMainState,
} from '../../../contexts';
import { BackendError, StateDispatch } from '../../../types';
import { useI18nError } from '../../../hooks';
import { routes } from '../../../router';
import { CCache } from '../../../cache';
import { PrivyLoginButton } from '../components';

type AddError = {
  error: string;
  details?: string;
};

function PassphraseLogin() {
  const [phrase, setPhrase] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<AddError | null>(null);

  const { daemonStatus, state } = useMainState();

  const { push } = useInAppNotify();
  const navigate = useNavigate();
  const { t } = useTranslation('login');
  const { tE } = useI18nError();
  const dispatch = useMainDispatch() as StateDispatch;

  const onChange = (phrase: string) => {
    setPhrase(phrase);
    if (phrase.length == 0) {
      setError(null);
    }
  };

  const handlePassphraseLogin = async () => {
    if (phrase.length === 0 || loading) {
      return;
    }
    if (state !== 'disconnected') {
      console.warn(`cannot login while tunnel state is ${state}`);
      return;
    }

    setLoading(true);
    try {
      console.info('logging in');
      await invoke<number | null>('add_account', { mnemonic: phrase.trim() });
      navigate(routes.root);
      dispatch({ type: 'set-account', stored: true });
      push({
        message: t('notification.added'),
        close: true,
      });
      await CCache.del('cache-account-id');
      await CCache.del('cache-device-id');
      dispatch({ type: 'reset-error' });
    } catch (e: unknown) {
      const eT = e as BackendError;
      setError({
        error: tE(eT.key),
        details: eT.data?.reason,
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <PageAnim className="h-full flex flex-col justify-end items-center gap-8 select-none cursor-default">
      <div className="text-center">
        <h1 className="text-3xl font-bold">{t('passphrase.title')}</h1>
        <p className="mt-6 text-iron dark:text-bombay">
          {t('passphrase.description')}
        </p>
      </div>
      <div className="flex flex-col gap-3 w-full">
        <TextArea
          value={phrase}
          onChange={onChange}
          spellCheck={false}
          resize="none"
          rows={5}
          label={t('passphrase.input-label')}
          placeholder={t('passphrase.input-placeholder')}
          className="sentry-ignore"
          data-testid="login-mnemonic-input"
        />
        {error ? (
          <motion.div
            initial={{ opacity: 0, x: -10 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.15, ease: 'easeInOut' }}
            className={clsx([
              'text-aphrodisiac overflow-y-scroll max-h-16 wrap-break-word',
              'select-text text-sm',
            ])}
            data-testid="login-error-message"
          >
            {error.error}
            {error.details && `: ${error.details}`}
          </motion.div>
        ) : (
          <div className="h-4" />
        )}
      </div>
      <Button
        onClick={handlePassphraseLogin}
        disabled={daemonStatus === 'down' || state !== 'disconnected'}
        className={clsx(
          'h-14',
          daemonStatus === 'down' &&
            'opacity-50 disabled:opacity-50 hover:opacity-50',
        )}
        spinner={loading}
        data-testid="login-submit-button"
      >
        {t('passphrase.login-button')}
      </Button>
      <PrivyLoginButton outline color="gray" />
      <p className="text-iron dark:text-bombay">
        {t('passphrase.new-to-nymvpn')}
      </p>

      <Button
        outline
        color="gray"
        onClick={() => {
          openUrl(NymVpnPricingUrl);
        }}
        className="group border border-iron dark:border-bombay hover:ring-0! dark:hover:ring-0!"
      >
        <span className="flex items-center gap-2 text-black dark:text-white group-hover:text-black/50 dark:group-hover:text-white/80">
          {t('create-account')} <MsIcon icon="open_in_new" />
        </span>
      </Button>
    </PageAnim>
  );
}

export default PassphraseLogin;
