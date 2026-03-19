import clsx from 'clsx';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Trans, useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { NymSplash } from '../../assets';
import { Button, ButtonText, Link, MsIcon, PageAnim } from '../../ui';
import { useMainDispatch, useMainState } from '../../contexts';
import { NymVpnPricingUrl, PrivacyPolicyUrl, ToSUrl } from '../../constants';
import { routes } from '../../router';
import { PrivyButton } from '../../components';
import { invoke } from '@tauri-apps/api/core';
import { TAccountMode } from '../../types/tauri';
import { useRef } from 'react';
import { useDeepLink } from '../../hooks/index';
import { CCache } from '../../cache/index';
import { StateDispatch } from '../../types/index';

function Login() {
  const { uiTheme } = useMainState();
  const { t, i18n } = useTranslation('login');
  const navigate = useNavigate();

  const timeoutIdRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const { startListening } = useDeepLink();

  const dispatch = useMainDispatch() as StateDispatch;

  const handleCreateAccount = async () => {
    const url = await invoke<string>('get_deep_link', {
      locale: i18n.language,
      kind: 'CreateAccount',
    });

    openUrl(
      url.replace(
        'https://nymcom-git-deploy-sandbox-nyx-network-staging.vercel.app',
        'http://localhost:3000',
      ),
    );

    try {
      const timeoutPromise = new Promise<never>((_, reject) => {
        timeoutIdRef.current = setTimeout(
          () => reject(new Error('Login timeout')),
          300000,
        );
      });

      const deeplinkurl = await Promise.race([
        startListening(),
        timeoutPromise,
      ]);

      await invoke('store_deeplink_account', {
        callbackUrl: deeplinkurl,
      });

      dispatch({ type: 'set-account', stored: true });
      // await refreshAccountMode();
      await CCache.del('cache-account-id');
      await CCache.del('cache-device-id');
      dispatch({ type: 'reset-error' });
    } catch (error) {
      console.error('Create account error: ', error);
    } finally {
      if (timeoutIdRef.current !== null) {
        clearTimeout(timeoutIdRef.current);
        timeoutIdRef.current = null;
      }
    }
  };

  return (
    <PageAnim className="relative h-full flex flex-col justify-end items-center gap-6 select-none cursor-default">
      <NymSplash
        className={clsx('w-32', uiTheme === 'dark' ? 'fill-white' : 'fill-ash')}
      />
      <h1 className="text-2xl mt-12">{t('signup.title')}</h1>
      <div className="flex flex-col">
        <div className="py-6">
          <h2>{t('signup.maximum-privacy.title')}</h2>
          <p className="mt-2 text-iron dark:text-bombay whitespace-pre-line">
            {t('signup.maximum-privacy.description')}
          </p>
          <Button
            // onClick={() => {
            //   openUrl(NymVpnPricingUrl);
            //   navigate(routes.login);
            // }}
            onClick={handleCreateAccount}
            className="mt-4"
          >
            <div className="flex items-center gap-2 whitespace-pre-wrap">
              {t('signup.create-account')} <MsIcon icon="open_in_new" />
            </div>
          </Button>
        </div>
        <div className="py-6 border-t border-iron dark:border-bombay">
          <h2>{t('privy.use-existing-login.title')}</h2>
          <p className="mt-2 mb-4 text-iron dark:text-bombay whitespace-pre-line">
            {t('privy.use-existing-login.description')}
          </p>
          <PrivyButton />
        </div>

        <div className="flex flex-row justify-center items-center">
          <span className="dark:text-white truncate">
            {t('signup.already-have-an-account.title')}
          </span>
          <ButtonText onClick={() => navigate(routes.login)} color="malachite">
            {t('signup.already-have-an-account.button')}
          </ButtonText>
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
