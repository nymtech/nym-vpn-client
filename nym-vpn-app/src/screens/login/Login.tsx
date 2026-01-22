import { invoke } from '@tauri-apps/api/core';
import clsx from 'clsx';
import { useState } from 'react';
import { Trans, useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { useNavigate } from 'react-router';
import { useMainDispatch, useMainState } from '../../contexts';
import { useI18nError } from '../../hooks';
import { routes } from '../../router';
import { BackendError, StateDispatch } from '../../types';
import { Button, Link, PageAnim, TextArea } from '../../ui';
import { CCache } from '../../cache';
import { NymVpnPricingUrl, PrivacyPolicyUrl, ToSUrl } from '../../constants';
import { NymSplash } from '../../assets/index';
import { PrivyButton } from '../../components';

type AddError = {
  error: string;
  details?: string;
};

function Login() {
  const [phrase, setPhrase] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<AddError | null>(null);

  const { daemonStatus, state, welcomeChecked, uiTheme, backendFlags } =
    useMainState();

  const navigate = useNavigate();
  const { t } = useTranslation('add-credential');
  const { tE } = useI18nError();
  const dispatch = useMainDispatch() as StateDispatch;

  const onChange = (phrase: string) => {
    setPhrase(phrase);
    if (phrase.length === 0) {
      setError(null);
    }
  };

  const handleClick = async () => {
    if (phrase.length === 0 || loading) {
      return;
    }
    // kinda overkill but who knows?
    if (state !== 'disconnected') {
      console.warn(`cannot login while tunnel state is ${state}`);
      return;
    }

    setLoading(true);
    try {
      console.info('logging in');
      await invoke<number | null>('add_account', { mnemonic: phrase.trim() });
      if (!welcomeChecked) {
        navigate(routes.welcome);
      } else {
        navigate(routes.root);
      }
      dispatch({ type: 'set-account', stored: true });
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
    <PageAnim className="relative h-full flex flex-col justify-end items-center gap-6 select-none cursor-default">
      <NymSplash
        className={clsx('w-32', uiTheme === 'dark' ? 'fill-white' : 'fill-ash')}
      />
      <h1 className="text-2xl mt-12">{t('title')}</h1>

      <div className="py-0 w-full flex flex-col gap-3">
        <div className="w-full">
          <p className="mb-8 text-left text-iron dark:text-bombay w-11/12">
            {t('description')}
          </p>
          <TextArea
            value={phrase}
            onChange={onChange}
            spellCheck={false}
            resize="none"
            rows={6}
            label={t('input-label')}
            placeholder={t('input-placeholder')}
            className="sentry-ignore"
            data-testid="login-mnemonic-input"
          />
          {error ? (
            <motion.div
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.15, ease: 'easeInOut' }}
              className={clsx([
                'text-aphrodisiac overflow-y-scroll max-h-16 mt-3 mb-3 wrap-break-word',
                'select-text',
              ])}
              data-testid="login-error-message"
            >
              {error.error}
              {error.details && `: ${error.details}`}
            </motion.div>
          ) : (
            <div className="h-4"></div>
          )}
        </div>
        <div className="w-full flex flex-col justify-center items-center gap-6 mb-2">
          <Button
            onClick={handleClick}
            disabled={daemonStatus === 'down' || state !== 'disconnected'}
            className={clsx(
              'h-14',
              daemonStatus === 'down' &&
                'opacity-50 disabled:opacity-50 hover:opacity-50',
            )}
            spinner={loading}
            data-testid="login-submit-button"
          >
            {t('login-button')}
          </Button>
          {backendFlags.privy && <PrivyButton />}
          <div
            className="text-sm flex flex-row justify-center items-center gap-2"
            data-testid="login-create-account-section"
          >
            <span
              className="dark:text-white truncate"
              data-testid="login-create-account-text"
            >
              {t('create-account.text')}
            </span>
            <Link
              text={t('create-account.link')}
              url={NymVpnPricingUrl}
              color="malachite"
            />
          </div>
        </div>
      </div>
      <p
        className="text-xs text-center text-iron dark:text-bombay w-80"
        data-testid="welcome-tos-notice"
      >
        <Trans
          i18nKey="tos-notice"
          ns="welcome"
          components={{
            tosLink: (
              <Link
                color="primary"
                text={t('tos', { ns: 'common' })}
                url={ToSUrl}
                textClassName="underline-offset-2"
                data-testid="welcome-tos-link"
              />
            ),
            privacyLink: (
              <Link
                color="primary"
                text={t('privacy-statement', { ns: 'common' })}
                url={PrivacyPolicyUrl}
                textClassName="underline-offset-2"
                data-testid="welcome-privacy-link"
              />
            ),
          }}
        />
      </p>
    </PageAnim>
  );
}

export default Login;
