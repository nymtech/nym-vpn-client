import { useTranslation } from 'react-i18next';
import { useMemo } from 'react';
import { ButtonIcon, CardNew, CardNewHeader, MsIcon } from '../../../../ui';
import { useMainState } from '../../../../store';
import { useToast } from '../../../../hooks';
import { NoActivePlan } from './NoActivePlan';
import { ActivePlan } from './ActivePlan';

export type AccountStatusProps = {
  refresh: () => Promise<void>;
  refreshing: boolean;
};

export function AccountStatus({ refresh, refreshing }: AccountStatusProps) {
  const { add } = useToast();
  const { t } = useTranslation('account');

  const { accountState, accountSummary, accountSyncing, daemonStatus } =
    useMainState();

  const needsSubscription = useMemo(
    () =>
      accountState === 'no-subscription' ||
      accountState === 'status-not-active' ||
      !accountSummary?.isSubscriptionActive,
    [accountState, accountSummary],
  );

  const handleRefresh = async () => {
    try {
      await refresh();
    } catch (error: unknown) {
      console.error('Failed to refresh account state: ', error);
      add({
        id: 'refresh-account-state-error',
        title: t('account-status.failed-to-refresh-account-state'),
        type: 'error',
      });
    }
  };

  if (!accountState || accountState === 'error' || accountState === 'offline') {
    return null;
  }

  return (
    <>
      <CardNew>
        <CardNewHeader className="border-text-tertiary/30 dark:border-surface-bg justify-between! border-b">
          <div className="flex flex-row items-center gap-2">
            <MsIcon icon="speed" className="text-text-secondary" />
            <p className="text-text-primary truncate text-left text-base select-none">
              {t('account-status.title')}
            </p>
          </div>
          <ButtonIcon
            icon="refresh"
            color="chalk"
            onClick={handleRefresh}
            disabled={daemonStatus === 'down' || accountSyncing || refreshing}
            clickFeedback
            noDefaultSize
            className="h-8 min-h-8 w-8 min-w-8"
            iconClassName="!text-xl"
            data-testid="refresh-account-summary"
            aria-label={t('account-status.refresh')}
          />
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
