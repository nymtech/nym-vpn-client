import { useTranslation } from 'react-i18next';
import { useMemo } from 'react';
import { clsx } from 'clsx';
import { Button } from '@headlessui/react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { CardNewFooter, MsIcon } from '../../../../ui';
import { TAccountSummary } from '../../../../types';
import { getAccountStatus } from '../utils';
import { NymVpnAccountLoginUrl } from '../../../../constants';

export function RenewButton({
  accountSummary,
}: {
  accountSummary: TAccountSummary;
}) {
  const { t } = useTranslation('account');

  const status = useMemo(
    () => getAccountStatus(accountSummary),
    [accountSummary],
  );

  const handleRenew = async () => {
    console.log('Renew now to stay protected');
    // temporary redirect to login page
    await openUrl(NymVpnAccountLoginUrl);
    // TODO: Implement renew logic
  };

  const getStatusColor = () => {
    if (status === 'green' || status === 'yellow') {
      return 'bg-malachite-moss/10 hover:bg-malachite-moss/20 dark:bg-malachite/10 dark:hover:bg-malachite/20 text-malachite-moss dark:text-malachite';
    }
    if (status === 'amber') {
      return 'bg-cheddar/10 hover:bg-cheddar/20 dark:bg-king-nacho/10 dark:hover:bg-king-nacho/20 text-cheddar dark:text-king-nacho';
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
        <MsIcon icon="open_in_new" />
      </Button>
    </CardNewFooter>
  );
}
