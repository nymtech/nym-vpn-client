import { useTranslation } from 'react-i18next';
import dayjs from 'dayjs';
import { useMemo } from 'react';
import { clsx } from 'clsx';
import { Button } from '@headlessui/react';
import { CardNewFooter, MsIcon } from '../../../../ui';
import { TAccountSummary } from '../../../../types';

export function RenewButton({
  accountSummary,
}: {
  accountSummary: TAccountSummary;
}) {
  const { t } = useTranslation('account');

  const status = useMemo(() => {
    const diff = dayjs
      .unix(Number(accountSummary['subscription-valid-until']))
      .diff(dayjs(), 'day');

    if (
      accountSummary['subscription-kind'] === 'freepass' ||
      accountSummary['subscription-kind'] === 'one-month'
    ) {
      if (diff < 3) return 'yellow'; // 2 days left
      if (diff < 8) return 'green'; // 7 days left
      return 'ok';
    }

    // 1 & 2 years subscriptions
    if (diff < 31) return 'yellow'; // 30 days left
    if (diff < 61) return 'green'; // 60 days left
    return 'ok';
  }, [accountSummary]);

  const handleRenew = () => {
    console.log('Renew now to stay protected');
    // TODO: Implement renew logic
  };

  const getStatusColor = () => {
    if (status === 'green') {
      return 'bg-malachite-moss/10 hover:bg-malachite-moss/20 dark:bg-malachite/10 dark:hover:bg-malachite/20 text-malachite-moss dark:text-malachite';
    }
    if (status === 'yellow') {
      return 'bg-cheddar/10 hover:bg-cheddar/20 dark:bg-king-nacho/10 dark:hover:bg-king-nacho/20 text-cheddar dark:text-king-nacho';
    }
  };

  if (status === 'ok') {
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
