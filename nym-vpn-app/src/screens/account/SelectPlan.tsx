import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button, MsIcon, PageAnim, Spinner } from '../../ui';
import { routes } from '../../router';
import { CheckCircleIcon } from '../../assets';
import { useAutologin } from '../../contexts/autologin/context';
import { useDeepLink, useToast } from '../../hooks';
import { DeeplinkTimeout } from '../../errors';

function Feature({ icon, title }: { icon: string; title: string }) {
  return (
    <h3 className="dark:text-bombay flex w-72 items-center gap-2 text-left text-base">
      <MsIcon icon={icon} className="text-malachite" /> {title}
    </h3>
  );
}

function SelectPlan() {
  const navigate = useNavigate();
  const { t } = useTranslation('account');

  const [autologinLoading, setAutologinLoading] = useState(false);
  const { autologin, closeDialog } = useAutologin();
  const { startListening } = useDeepLink();
  const { add } = useToast();

  const handleClick = async () => {
    if (autologinLoading) return;

    setAutologinLoading(true);
    try {
      await autologin('autologinRenew');

      await startListening(600000);

      await invoke<void>('handle_subscription_payment');
      closeDialog();
      navigate(routes.root);
    } catch (error: unknown) {
      console.error('Select plan error: ', error);
      if (error instanceof DeeplinkTimeout) {
        add({
          title: t('autologin.timeout', { ns: 'errors' }),
          type: 'error',
        });
      }
    } finally {
      setAutologinLoading(false);
    }
  };

  return (
    <PageAnim className="flex h-full cursor-default flex-col items-center select-none">
      <div className="flex grow flex-col items-center justify-center gap-6 px-4">
        <CheckCircleIcon className="text-malachite text-4xl" />
        <div className="flex flex-col gap-2 text-center text-2xl dark:text-white">
          <h1 className="truncate">{t('select-a-plan.title')}</h1>
        </div>
        <div className="flex flex-col gap-2 self-start">
          <Feature
            icon="verified_user"
            title={t('select-a-plan.features.all-included')}
          />
          <Feature icon="campaign" title={t('select-a-plan.features.no-ads')} />
          <Feature
            icon="chat_error"
            title={t('select-a-plan.features.cancel-anytime')}
          />
        </div>
      </div>
      <Button className="w-full" onClick={handleClick}>
        <span className="flex items-center gap-2">
          {t('select-a-plan.button')}{' '}
          {autologinLoading ? <Spinner /> : <MsIcon icon="open_in_new" />}
        </span>
      </Button>
    </PageAnim>
  );
}

export default SelectPlan;
