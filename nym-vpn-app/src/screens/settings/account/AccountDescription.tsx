import { useTranslation } from 'react-i18next';
import { useMemo } from 'react';
import dayjs from 'dayjs';
import clsx from 'clsx';
import { useMainState } from '../../../contexts';
import {
  getAccountDescriptionColor,
  getAccountStateDescription,
  getAccountStatus,
} from './utils';

const statusColors = {
  amber: 'text-liquid-lava',
  yellow: 'text-cheddar dark:text-king-nacho',
  green: 'text-malachite-moss dark:text-malachite',
} as const;

export function AccountDescription() {
  const { t } = useTranslation('settings');
  const { accountSyncing, accountState, accountSummary } = useMainState();
  console.log('[AccountDescription] accountSummary', accountSummary);
  console.log('[AccountDescription] accountState', accountState);

  const accountStateDescription = getAccountStateDescription(
    t,
    accountSyncing,
    accountState,
  );

  const status = useMemo(
    () => getAccountStatus(accountSummary),
    [accountSummary],
  );

  if (accountStateDescription) {
    return (
      <span
        className={clsx(
          getAccountDescriptionColor(accountSyncing, accountState),
        )}
      >
        {accountStateDescription}
      </span>
    );
  }

  if (!accountSummary) {
    return null;
  }

  return (
    <>
      <p className={statusColors[status]}>
        {t('account.planValidUntil', {
          date: dayjs
            .unix(Number(accountSummary?.['subscription-valid-until']))
            .format('MMMM D, YYYY'),
        })}
      </p>
      {accountSummary?.['is-recurring'] && (
        <p className="text-iron dark:text-bombay">
          *{t('account.auto-renews')}
        </p>
      )}
    </>
  );
}
