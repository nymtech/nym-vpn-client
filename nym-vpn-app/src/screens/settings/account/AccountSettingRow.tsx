import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useMemo } from 'react';
import clsx from 'clsx';
import dayjs from 'dayjs';
import { AccountState, StateDispatch } from '../../../types';
import { useMainDispatch, useMainState } from '../../../contexts';
import { Button, MsIcon } from '../../../ui';
import { routes } from '../../../router';
import SettingsGroup from '../SettingsGroup';
import { getAccountColor, getAccountDescription } from './utils';

export function AccountDescription() {
  const { t } = useTranslation('settings');
  const { accountSyncing, accountState, accountSummary } = useMainState();
  console.log('[AccountDescription] accountSummary', accountSummary);
  console.log('[AccountDescription] accountState', accountState);

  const desc = getAccountDescription(t, accountSyncing, accountState);

  const status = useMemo(() => {
    const diff = dayjs
      .unix(Number(accountSummary?.['subscription-valid-until']))
      .diff(dayjs(), 'day');

    if (
      accountSummary?.['subscription-kind'] === 'freepass' ||
      accountSummary?.['subscription-kind'] === 'one-month'
    ) {
      if (diff < 3) return 'amber'; // 2 days left
      if (diff < 8) return 'yellow'; // 7 days left
      return 'green';
    }

    // 1 & 2 years subscriptions
    if (diff < 31) return 'amber'; // 30 days left
    if (diff < 61) return 'yellow'; // 60 days left
    return 'green';
  }, [accountSummary]);

  const getStatusColor = () => {
    switch (status) {
      case 'amber':
        return 'text-liquid-lava';
      case 'yellow':
        return 'text-cheddar dark:text-king-nacho';
      case 'green':
        return 'text-malachite-moss dark:text-malachite';
    }
  };

  if (desc) {
    return (
      <span className={clsx(getAccountColor(accountSyncing, accountState))}>
        {desc}
      </span>
    );
  }

  if (!accountSummary) {
    return null;
  }

  return (
    <>
      <p className={getStatusColor()}>
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

function AccountSettingRow() {
  const { daemonStatus, account, accountState, accountSyncing } =
    useMainState();

  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');
  const needAPlan =
    account &&
    (accountState === 'no-subscription' ||
      accountState === 'bandwidth-exceeded');

  useEffect(() => {
    const checkAccount = async () => {
      try {
        const stored = await invoke<boolean | undefined>('is_account_stored');
        dispatch({ type: 'set-account', stored: stored || false });
      } catch {}
    };

    if (daemonStatus !== 'down') {
      checkAccount();
    }
  }, [daemonStatus, dispatch]);

  if (!account) {
    return (
      <Button
        onClick={() => navigate(routes.onboarding)}
        disabled={daemonStatus === 'down'}
      >
        {t('account.get-started')}
      </Button>
    );
  }

  return (
    <>
      {needAPlan && (
        <Button
          onClick={() => navigate(routes.selectPlan)}
          disabled={daemonStatus === 'down' || accountSyncing}
        >
          {t('account.choose-plan')}
        </Button>
      )}
      <SettingsGroup
        settings={[
          {
            title: 'Account',
            desc: <AccountDescription />,
            // desc: (
            //   <span
            //     className={clsx(getAccountColor(accountSyncing, accountState))}
            //   >
            //     {/* {getAccountDescription(t, accountSyncing, accountState)} */}
            //   </span>
            // ),
            leadingIcon: 'account_circle',
            onClick: () => navigate(routes.accountSettings),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
        ]}
      />
    </>
  );
}

export default AccountSettingRow;
