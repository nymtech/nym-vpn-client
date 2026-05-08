import { useTranslation } from 'react-i18next';
import { useMemo } from 'react';
import { CardNew, CardNewHeader, MsIcon } from '../../../../ui';
import { useMainState } from '../../../../store';
import { NoActivePlan } from './NoActivePlan';
import { ActivePlan } from './ActivePlan';

export function AccountStatus() {
  const { t } = useTranslation('account');

  const { accountState, accountSummary } = useMainState();

  const needsSubscription = useMemo(
    () =>
      accountState === 'no-subscription' ||
      accountState === 'status-not-active' ||
      !accountSummary?.isSubscriptionActive,
    [accountState, accountSummary],
  );

  if (!accountState || accountState === 'error' || accountState === 'offline') {
    return null;
  }

  return (
    <>
      <CardNew>
        <CardNewHeader className="border-bombay/30 dark:border-ash border-b">
          <div className="flex flex-row items-center gap-2">
            <MsIcon icon="speed" className="text-text-secondary" />
            <p className="text-text-primary truncate text-left text-base select-none">
              {t('account-status.title')}
            </p>
          </div>
        </CardNewHeader>
        {needsSubscription ||
        accountState === 'pending-subscription' ||
        !accountSummary ? (
          <NoActivePlan />
        ) : (
          <ActivePlan accountSummary={accountSummary} />
        )}
      </CardNew>
    </>
  );
}
