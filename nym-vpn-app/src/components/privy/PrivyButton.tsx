import { openUrl } from '@tauri-apps/plugin-opener';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useNavigate } from 'react-router';
import { Button, MsIcon } from '../../ui';
import { useMainDispatch, useMainState } from '../../contexts';
import { useDeepLink } from '../../hooks';
import { routes } from '../../router';
import { CCache } from '../../cache';
import { StateDispatch, TAccountMode } from '../../types';
import { DeeplinkTimeout } from '../../errors';
import { useNewToast } from '../../contexts/new-toast-provider/index';

function PrivyButton() {
  const { t, i18n } = useTranslation('login');

  const { add } = useNewToast();
  const { startListening } = useDeepLink();
  const { welcomeChecked } = useMainState();
  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;

  const [loading, setLoading] = useState(false);

  const refreshAccountMode = async () => {
    const accountMode = await invoke<TAccountMode>('get_account_mode');
    dispatch({ type: 'set-account-mode', mode: accountMode });
  };

  const handlePrivy = async () => {
    setLoading(true);

    const loginUrl = await invoke<string>('get_deep_link', {
      locale: i18n.language,
      kind: 'Privy',
    });
    openUrl(loginUrl);

    try {
      const deeplinkurl = await startListening(300000);

      await invoke('store_deeplink_account', {
        callbackUrl: deeplinkurl,
      });

      if (!welcomeChecked) {
        navigate(routes.welcome);
      } else {
        navigate(routes.root);
      }

      dispatch({ type: 'set-account', stored: true });
      await refreshAccountMode();
      await CCache.del('cache-account-id');
      await CCache.del('cache-device-id');
      dispatch({ type: 'reset-error' });
    } catch (error) {
      console.error('Privy login error: ', error);
      if (error instanceof DeeplinkTimeout) {
        add({
          title: t('privy.error.timeout'),
          description: t('privy.error.timeout'),
          type: 'error',
        });
      } else {
        add({
          title: t('privy.error.login'),
          description: t('privy.error.login'),
          type: 'error',
        });
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <Button
      outline
      color="gray"
      onClick={handlePrivy}
      className="group border border-iron dark:border-bombay hover:ring-0! dark:hover:ring-0!"
      spinner={loading}
    >
      <span className="flex items-center gap-2 whitespace-pre-wrap text-black dark:text-white group-hover:text-black/50 dark:group-hover:text-white/80">
        {t('privy.login-button')} <MsIcon icon="open_in_new" />
      </span>
    </Button>
  );
}

export default PrivyButton;
