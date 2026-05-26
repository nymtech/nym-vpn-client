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
      return 'bg-brand-primary/10 hover:bg-brand-primary/20 text-brand-primary';
    }
    if (status === 'amber') {
      return 'bg-status-warning/10 hover:bg-status-warning/20 text-status-warning';
    }
  };

  const getSpinnerColor = () => {
    if (status === 'green' || status === 'yellow') {
      return 'border-brand-primary';
    }
    if (status === 'amber') {
      return 'border-status-warning';
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
