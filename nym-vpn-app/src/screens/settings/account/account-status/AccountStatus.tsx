import { Trans, useTranslation } from 'react-i18next';
import { useMemo } from 'react';
import { CardNew, CardNewHeader, Link, MsIcon } from '../../../../ui';
import { useMainState } from '../../../../contexts';
import { ContactSupportUrl } from '../../../../constants';
import { NoActivePlan } from './NoActivePlan';
import { ActivePlan } from './ActivePlan';

export function AccountStatus() {
  const { t } = useTranslation('account');

  const { accountState, accountSummary } = useMainState();

  const needsSubscription = useMemo(
    () =>
      accountState === 'no-subscription' ||
      accountState === 'status-not-active',
    [accountState],
  );

  if (!accountState || accountState === 'error' || accountState === 'offline') {
    return null;
  }

  return (
    <>
      <CardNew>
        <CardNewHeader className="border-b border-bombay/30 dark:border-ash">
          <div className="flex flex-row items-center gap-2">
            <MsIcon icon="speed" className="text-iron dark:text-bombay" />
            <p className="text-left truncate text-base text-baltic-sea dark:text-white select-none">
              {t('account-status.title')}
            </p>
          </div>
        </CardNewHeader>
        {needsSubscription || !accountSummary ? (
          <NoActivePlan />
        ) : (
          <ActivePlan accountSummary={accountSummary} />
        )}
      </CardNew>
      {!needsSubscription && !!accountSummary && (
        <p className="text-sm text-iron dark:text-bombay">
          <Trans
            i18nKey="account-status.contact-support"
            ns="account"
            components={{
              1: <Link color="primary" url={ContactSupportUrl} />,
            }}
          />
        </p>
      )}
    </>
  );
}
