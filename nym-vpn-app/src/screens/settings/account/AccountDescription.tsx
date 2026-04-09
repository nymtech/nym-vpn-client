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

  const accountStateDescription = getAccountStateDescription(
    t,
    accountSyncing,
    accountState,
    accountSummary,
  );

  const status = useMemo(
    () => getAccountStatus(accountSummary),
    [accountSummary],
  );

  if (accountStateDescription) {
    return (
      <span
        className={clsx(
          getAccountDescriptionColor(
            accountSyncing,
            accountState,
            accountSummary,
          ),
        )}
      >
        {accountStateDescription}
      </span>
    );
  }

  if (!accountSummary?.subscription?.subscription?.['valid-until-utc']) {
    return null;
  }

  return (
    <>
      <p className={statusColors[status]}>
        {t('account.planValidUntil', {
          date: dayjs
            .unix(
              Number(
                accountSummary?.subscription?.subscription?.['valid-until-utc'],
              ),
            )
            .format('MMMM D, YYYY'),
        })}
      </p>
      {accountSummary?.subscription?.subscription?.['is-recurring'] && (
        <p className="text-iron dark:text-bombay">
          *{t('account.auto-renews')}
        </p>
      )}
    </>
  );
}
