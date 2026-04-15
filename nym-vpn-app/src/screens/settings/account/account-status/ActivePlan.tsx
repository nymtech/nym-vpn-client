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

    return ((Number(limit) - Number(used)) / Number(limit)) * 100;
  }, [accountSummary]);

  const bandwidthRemainingValue = useMemo(
    () =>
      formatGb(accountSummary.trafficLimitGb - accountSummary.trafficUsedGb),
    [accountSummary],
  );

  const resetsOn = useMemo(() => {
    return dayjs
      .unix(Number(accountSummary.trafficResetTime))
      .format('DD.MM.YYYY');
  }, [accountSummary]);

  return (
    <>
      <CardNewBody className="py-5">
        <Progress.Root
          className="grid w-full grid-cols-2 gap-y-2"
          value={bandwidthRemainingProgress}
        >
          <Progress.Label className="text-sm font-medium text-malachite-moss dark:text-malachite">
            {t('account-status.bandwidth-remaining')}
          </Progress.Label>
          <Progress.Label className="text-sm font-medium text-right text-iron dark:text-bombay">
            {t('account-status.limit')}
          </Progress.Label>
          <Progress.Track className="col-span-full h-1 overflow-hidden rounded bg-mercury dark:bg-ash">
            <Progress.Indicator className="block bg-malachite-moss dark:bg-malachite transition-all duration-500" />
          </Progress.Track>
          <Progress.Label className="text-sm font-medium text-malachite-moss dark:text-malachite">
            {bandwidthRemainingValue}
          </Progress.Label>
          <Progress.Label className="text-sm font-medium text-right text-iron dark:text-bombay">
            {formatGb(accountSummary.trafficLimitGb)}
          </Progress.Label>
        </Progress.Root>
        <Separator
          orientation="horizontal"
          className="w-full h-px bg-mercury dark:bg-ash"
        />
        <div className="flex justify-between items-center w-full pt-3">
          <p className="text-sm text-iron dark:text-bombay select-none">
            {t('account-status.resets-on')}
          </p>
          <p className="text-sm text-baltic-sea dark:text-white font-mono select-none">
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
