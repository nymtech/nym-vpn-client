import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Separator } from '@base-ui-components/react/separator';
import { Progress } from '@base-ui-components/react/progress';
import dayjs from 'dayjs';
import { TAccountSummary } from '../../../../types';
import { formatGb } from '../../../../util';
import { CardNewBody } from '../../../../ui';
import { RenewButton } from './RenewButton';

export function ActivePlan({
  accountSummary,
}: {
  accountSummary: TAccountSummary;
}) {
  const { t } = useTranslation('account');

  const dataUnavailable = accountSummary.fairUsageDataUnavailable;

  const bandwidthProgress = useMemo(() => {
    const used = Number(accountSummary.trafficUsedGb);
    const limit = Number(accountSummary.trafficLimitGb);
    if (!Number.isFinite(used) || !Number.isFinite(limit) || limit <= 0) {
      return 0;
    }
    return Math.min((used / limit) * 100, 100);
  }, [accountSummary]);

  // trafficResetTime is a unix timestamp (UTC-sourced); dayjs.unix renders it
  // in the user's local timezone by default.
  const resetsOn = useMemo(() => {
    const ts = accountSummary.trafficResetTime;
    if (ts === null) {
      return null;
    }
    const date = dayjs.unix(Number(ts));
    return date.isValid() ? date.format('D MMMM YYYY') : null;
  }, [accountSummary]);

  return (
    <>
      <CardNewBody className="py-5">
        {dataUnavailable ? (
          <p className="text-text-secondary w-full py-2 text-sm select-none">
            {t('account-status.data-unavailable')}
          </p>
        ) : (
          <>
            <Progress.Root
              className="grid w-full grid-cols-2 gap-y-2"
              value={bandwidthProgress}
            >
              <Progress.Label className="text-brand-primary text-sm font-medium">
                {t('account-status.daily-allowance-used')}
              </Progress.Label>
              <Progress.Label className="text-text-secondary text-right text-sm font-medium">
                {t('account-status.daily-limit')}
              </Progress.Label>
              <Progress.Track className="bg-surface-elev dark:bg-surface-bg col-span-full h-1 overflow-hidden rounded">
                <Progress.Indicator className="bg-brand-primary block transition-all duration-500" />
              </Progress.Track>
              <Progress.Label className="text-brand-primary text-sm font-medium">
                {formatGb(accountSummary.trafficUsedGb)}
              </Progress.Label>
              <Progress.Label className="text-text-secondary text-right text-sm font-medium">
                {formatGb(accountSummary.trafficLimitGb)}
              </Progress.Label>
            </Progress.Root>
            <p className="text-text-tertiary w-full pt-2 text-xs select-none">
              {t('account-status.bandwidth-helper')}
            </p>
          </>
        )}
        {/* Reset row is shown regardless of dataUnavailable: the daily reset
            schedule is still valid even when usage figures are missing. */}
        <Separator
          orientation="horizontal"
          className="bg-surface-elev dark:bg-surface-bg h-px w-full"
        />
        <div className="flex w-full items-center justify-between pt-3">
          <p className="text-text-secondary text-sm select-none">
            {t('account-status.resets-daily')}
          </p>
          <p className="text-text-primary font-mono text-sm select-none">
            {resetsOn ?? t('account-status.reset-unknown')}
          </p>
        </div>
      </CardNewBody>
      {(!accountSummary.subscription?.subscription?.isRecurring ||
        !accountSummary.isSubscriptionStacked) && (
        <RenewButton accountSummary={accountSummary} />
      )}
    </>
  );
}
