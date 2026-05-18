import { openUrl } from '@tauri-apps/plugin-opener';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useNavigate } from 'react-router';
import { ButtonNew, MsIcon } from '../../ui';
import { dispatch, useAppStore } from '../../store';
import { useDeepLink, useToast } from '../../hooks';
import { routes } from '../../router';
import { CCache } from '../../cache';
import { TAccountMode } from '../../types';
import { DeeplinkTimeout } from '../../errors';

function PrivyButton({ label }: { label: string }) {
  const { t, i18n } = useTranslation('login');

  const { add } = useToast();
  const { startListening } = useDeepLink();
  const technicalOptinSeen = useAppStore((state) => state.technicalOptinSeen);
  const navigate = useNavigate();

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

      if (!technicalOptinSeen) {
        navigate(routes.technicalOptin);
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
          type: 'error',
        });
      } else {
        add({
          title: t('privy.error.login'),
          type: 'error',
        });
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <ButtonNew
      variant="outlined"
      onClick={handlePrivy}
      className="group border-iron dark:border-bombay border hover:ring-0! dark:hover:ring-0!"
      loading={loading}
    >
      <span className="flex items-center gap-2 whitespace-pre-wrap text-black group-hover:text-black/50 dark:text-white dark:group-hover:text-white/80">
        {label} <MsIcon icon="open_in_new" />
      </span>
    </ButtonNew>
  );
}

export default PrivyButton;
