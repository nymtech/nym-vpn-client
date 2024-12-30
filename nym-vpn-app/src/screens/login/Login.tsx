import { invoke } from '@tauri-apps/api/core';
import clsx from 'clsx';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'framer-motion';
import { useNavigate } from 'react-router';
import { NymDarkOutlineIcon, NymIcon } from '../../assets';
import { useInAppNotify, useMainDispatch, useMainState } from '../../contexts';
import { useI18nError } from '../../hooks';
import { routes } from '../../router';
import { BackendError, StateDispatch } from '../../types';
import { Button, Link, PageAnim, TextArea } from '../../ui';
import { MCache } from '../../cache';

type AddError = {
  error: string;
  details?: string;
};

function Login() {
  const { uiTheme, daemonStatus, accountLinks } = useMainState();
  const [phrase, setPhrase] = useState('');
  const [error, setError] = useState<AddError | null>(null);

  const { push } = useInAppNotify();
  const navigate = useNavigate();
  const { t } = useTranslation('addCredential');
  const { tE } = useI18nError();
  const dispatch = useMainDispatch() as StateDispatch;
  const signUpUrl = accountLinks?.signUp;

  const onChange = (phrase: string) => {
    setPhrase(phrase);
    if (phrase.length == 0) {
      setError(null);
    }
  };

  const handleClick = () => {
    if (phrase.length === 0) {
      return;
    }
    invoke<number | null>('add_account', { mnemonic: phrase.trim() })
      .then(() => {
        navigate(routes.root);
        dispatch({ type: 'set-account', stored: true });
        push({
          text: t('added-notification'),
          position: 'top',
          closeIcon: true,
        });
        MCache.del('account-id');
        MCache.del('device-id');
      })
      .catch((e: unknown) => {
        const eT = e as BackendError;
        console.info('backend error:', e);
        setError({
          error: tE(eT.key),
          details: eT.data?.reason,
        });
      });
  };

  return (
    <PageAnim className="h-full flex flex-col justify-end items-center gap-10 select-none cursor-default">
      {uiTheme === 'Dark' ? (
        <NymDarkOutlineIcon className="w-22 h-22" />
      ) : (
        <NymIcon className="w-24 h-24 fill-ghost" />
      )}
      <div className="flex flex-col items-center gap-4 px-4">
        <h1 className="text-2xl dark:text-white">{t('welcome')}</h1>
        <h2 className="text-center text-dim-gray dark:text-mercury-mist w-11/12">
          {t('description')}
        </h2>
      </div>
      <div className="w-full">
        <TextArea
          value={phrase}
          onChange={onChange}
          spellCheck={false}
          resize="none"
          rows={5}
          label={t('input-label')}
          placeholder={t('input-placeholder')}
          className="sentry-ignore"
        />
        {error ? (
          <motion.div
            initial={{ opacity: 0, x: -10 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.15, ease: 'easeInOut' }}
            className={clsx([
              'text-teaberry overflow-y-scroll max-h-16 mt-3 break-words',
              'select-text',
            ])}
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
          disabled={daemonStatus === 'NotOk'}
          className={clsx(
            daemonStatus === 'NotOk' &&
              'opacity-50 disabled:opacity-50 hover:opacity-50',
          )}
        >
          {t('login-button')}
        </Button>
        {signUpUrl && (
          <div className="flex flex-row justify-center items-center gap-2">
            <span className="dark:text-mercury-pinkish truncate">
              {t('create-account.text')}
            </span>
            <Link text={t('create-account.link')} url={signUpUrl} />
          </div>
        )}
      </div>
    </PageAnim>
  );
}

export default Login;
