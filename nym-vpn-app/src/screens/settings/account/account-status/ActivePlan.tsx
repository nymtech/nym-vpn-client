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

  const bandwidthRemainingProgress = useMemo(() => {
    const used = accountSummary.trafficUsedGb;
    const limit = accountSummary.trafficLimitGb;

    return (Number(used) / Number(limit)) * 100;
  }, [accountSummary]);

  const resetsOn = useMemo(() => {
    return dayjs
      .unix(Number(accountSummary.trafficResetTime))
      .format('D MMMM YYYY');
  }, [accountSummary]);

  return (
    <>
      <CardNewBody className="py-5">
        <Progress.Root
          className="grid w-full grid-cols-2 gap-y-2"
          value={bandwidthRemainingProgress}
        >
          <Progress.Label className="text-primary text-sm font-medium">
            {t('account-status.bandwidth-remaining')}
          </Progress.Label>
          <Progress.Label className="text-text-secondary text-right text-sm font-medium">
            {t('account-status.limit')}
          </Progress.Label>
          <Progress.Track className="bg-mercury dark:bg-ash col-span-full h-1 overflow-hidden rounded">
            <Progress.Indicator className="bg-primary block transition-all duration-500" />
          </Progress.Track>
          <Progress.Label className="text-primary text-sm font-medium">
            {formatGb(accountSummary.trafficUsedGb)}
          </Progress.Label>
          <Progress.Label className="text-text-secondary text-right text-sm font-medium">
            {formatGb(accountSummary.trafficLimitGb)}
          </Progress.Label>
        </Progress.Root>
        <Separator
          orientation="horizontal"
          className="bg-mercury dark:bg-ash h-px w-full"
        />
        <div className="flex w-full items-center justify-between pt-3">
          <p className="text-text-secondary text-sm select-none">
            {t('account-status.resets-on')}
          </p>
          <p className="text-text-primary font-mono text-sm select-none">
            {resetsOn}
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
