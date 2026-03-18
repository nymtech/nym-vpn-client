import { useTranslation } from 'react-i18next';
import { useMemo } from 'react';
import { clsx } from 'clsx';
import { Button } from '@headlessui/react';
import { CardNewFooter, MsIcon, Spinner } from '../../../../ui';
import { TAccountSummary } from '../../../../types';
import { getAccountStatus } from '../utils';
import { useAutologin } from '../../../../contexts';

export function RenewButton({
  accountSummary,
}: {
  accountSummary: TAccountSummary;
}) {
  const { t } = useTranslation('account');

  const { autologin, autologinLoading } = useAutologin();

  const status = useMemo(
    () => getAccountStatus(accountSummary),
    [accountSummary],
  );

  const handleRenew = async () => {
    await autologin('autologinRenew');
  };

  const getStatusColor = () => {
    if (status === 'green' || status === 'yellow') {
      return 'bg-malachite-moss/10 hover:bg-malachite-moss/20 dark:bg-malachite/10 dark:hover:bg-malachite/20 text-malachite-moss dark:text-malachite';
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
          'flex flex-row items-center justify-between w-full py-3 px-5 rounded-b-lg',
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
