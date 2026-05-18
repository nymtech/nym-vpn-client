import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useCallback } from 'react';
import { dispatch, useAppStore } from '../../../store';
import { ButtonNew } from '../../../ui';
import { useDeepLink } from '../../../hooks';
import { CCache } from '../../../cache';
import { routes } from '../../../router';
import { PrivyButton } from '../../../components';

export function Signup() {
  const { t, i18n } = useTranslation('login');
  const navigate = useNavigate();
  const technicalOptinSeen = useAppStore((state) => state.technicalOptinSeen);

  const { startListening } = useDeepLink();

  const handleNavigate = useCallback(() => {
    if (!technicalOptinSeen) {
      navigate(routes.technicalOptin);
    } else {
      navigate(routes.root);
    }
  }, [technicalOptinSeen, navigate]);

  const handleCreateAccount = async () => {
    const url = await invoke<string>('get_deep_link', {
      locale: i18n.language,
      kind: 'CreateAccount',
    });

    openUrl(url);

    try {
      const deeplinkurl = await startListening(600000);

      await invoke('store_deeplink_account', {
        callbackUrl: deeplinkurl,
      });

      dispatch({ type: 'set-account', stored: true });

      await CCache.del('cache-account-id');
      await CCache.del('cache-device-id');
      dispatch({ type: 'reset-error' });

      handleNavigate();
    } catch (error) {
      console.error('[Signup] Create account error: ', error);
      // if error, then most likely the deeplink call timed out.
      // But the user might still finish the purchase on the website.
      handleNavigate();
    }
  };

  return (
    <div className="flex h-full flex-col items-center justify-between gap-6">
      <div className="flex flex-col items-center gap-2">
        <h1 className="text-text-primary text-2xl font-medium tracking-tight">
          {t('signup.title')}
        </h1>
      </div>
      <div className="flex w-full flex-col gap-3">
        <ButtonNew onClick={handleCreateAccount}>
          {t('signup.signup-anonymous-button')}
        </ButtonNew>
        <PrivyButton label={t('signup.signup-social-button')} />
        <p className="text-bombay text-center text-xs leading-5">
          {t('signup.disclaimer')}
        </p>
      </div>
    </div>
  );
}
