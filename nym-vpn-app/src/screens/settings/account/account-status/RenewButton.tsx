import { useTranslation } from 'react-i18next';
import { useMemo, useState } from 'react';
import { clsx } from 'clsx';
import { Button } from '@headlessui/react';
import { invoke } from '@tauri-apps/api/core';
import { CardNewFooter, MsIcon, Spinner } from '../../../../ui';
import { TAccountSummary } from '../../../../types';
import { getAccountStatus } from '../utils';
import { useAutologin } from '../../../../contexts';
import { useDeepLink, useToast } from '../../../../hooks';
import { DeeplinkTimeout } from '../../../../errors';

export function RenewButton({
  accountSummary,
}: {
  accountSummary: TAccountSummary;
}) {
  const { t } = useTranslation('account');

  const [autologinLoading, setAutologinLoading] = useState(false);
  const { autologin, closeDialog } = useAutologin();
  const { startListening } = useDeepLink();
  const { add } = useToast();

  const status = useMemo(
    () => getAccountStatus(accountSummary),
    [accountSummary],
  );

  const handleRenew = async () => {
    setAutologinLoading(true);
    try {
      await autologin('autologinRenew');

      await startListening(600000);

      await invoke<void>('handle_subscription_payment');
      closeDialog();
    } catch (error: unknown) {
      console.error('Renew button error: ', error);
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

  const getStatusColor = () => {
    if (status === 'green' || status === 'yellow') {
      return 'bg-malachite-moss/10 hover:bg-malachite-moss/20 dark:bg-malachite/10 dark:hover:bg-malachite/20 text-primary';
    }
    if (status === 'amber') {
      return 'bg-cheddar/10 hover:bg-cheddar/20 dark:bg-king-nacho/10 dark:hover:bg-king-nacho/20 text-cheddar dark:text-king-nacho';
    }
  };

  const getSpinnerColor = () => {
    if (status === 'green' || status === 'yellow') {
      return 'border-malachite-moss dark:border-malachite';
    }
    if (status === 'amber') {
      return 'border-cheddar dark:border-king-nacho';
    }
  };

  if (status === 'green') {
    return null;
  }

  return (
    <CardNewFooter className="p-0!">
      <Button
        className={clsx(
          'flex w-full flex-row items-center justify-between rounded-b-lg px-5 py-3',
          getStatusColor(),
        )}
        onClick={handleRenew}
      >
        <div className="flex flex-row items-center gap-2">
          <MsIcon icon="electric_bolt" />
          <p>{t('account-status.renew-now')}</p>
        </div>
        {autologinLoading ? (
          <Spinner className={getSpinnerColor()} />
        ) : (
          <MsIcon icon="open_in_new" />
        )}
      </Button>
    </CardNewFooter>
  );
}
