import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useEffect } from 'react';
import clsx from 'clsx';
import { StateDispatch } from '../../../types';
import { useMainDispatch, useMainState } from '../../../contexts';
import { Button, MsIcon } from '../../../ui';
import { routes } from '../../../router';
import SettingsGroup from '../SettingsGroup';
import { getAccountColor, getAccountDescription } from './utils';

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
            desc: (
              <span
                className={clsx(getAccountColor(accountSyncing, accountState))}
              >
                {getAccountDescription(t, accountSyncing, accountState)}
              </span>
            ),
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
