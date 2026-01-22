import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { routes } from '../../../router';
import { useMainDispatch, useMainState } from '../../../contexts';
import { AccountState, StateDispatch } from '../../../types';
import { Button, SettingsMenuCard } from '../../../ui';
import { capFirst } from '../../../util';

function Account() {
  const { daemonStatus, account, accountState, accountSyncing, accountLinks } =
    useMainState();

  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');
  const accountUrl = accountLinks?.account;
  const accountLoginUrl = accountLinks?.signIn;
  const needAPlan =
    account &&
    (accountState === 'no-subscription' ||
      accountState === 'bandwidth-exceeded');

  useEffect(() => {
    const checkAccount = async () => {
      try {
        const stored = await invoke<boolean | undefined>('is_account_stored');
        dispatch({ type: 'set-account', stored: stored || false });
      } catch { }
    };

    if (daemonStatus !== 'down') {
      checkAccount();
    }
  }, [daemonStatus, dispatch]);

  const handleGoToAccount = () => {
    if (accountUrl) {
      openUrl(accountUrl);
    } else if (accountLoginUrl) {
      openUrl(accountLoginUrl);
    }
  };

  const getAccountDescription = (state?: AccountState | null) => {
    if (!state) {
      return null;
    }
    if (accountSyncing) {
      return t('account.syncing');
    }
    switch (state) {
      case 'no-subscription':
        return t('account.no-plan');
      case 'max-device-reached':
        return t('account.max-device-reached');
      case 'status-not-active':
        return t('account.status-inactive');
      case 'bandwidth-exceeded':
        return t('account.bandwidth-exceeded');
      case 'requesting-zk-nyms':
        return t('account.requesting-zknyms');
      case 'offline':
      case 'error':
        return t('account.error');
      default:
        return null;
    }
  };

  const getAccountColor = (state?: AccountState | null) => {
    if (accountSyncing) {
      return 'normal';
    }
    if (
      state === 'no-subscription' ||
      state === 'bandwidth-exceeded' ||
      state === 'max-device-reached' ||
      state === 'error'
    ) {
      return 'red';
    }
    if (state === 'offline' || state === 'status-not-active') {
      return 'yellow';
    }
    return 'normal';
  };

  const getAccountButtonText = () => {
    if (needAPlan) {
      return t('account.choose-plan');
    }
    return t('account.get-started');
  };

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
          {getAccountButtonText()}
        </Button>
      )}
      <SettingsMenuCard
        title={capFirst(t('account', { ns: 'glossary' }))}
        onClick={handleGoToAccount}
        description={getAccountDescription(accountState) as string | undefined}
        descriptionColor={getAccountColor(accountState)}
        leadingIcon="account_circle"
        trailingIcon="open_in_new"
        disabled={!accountLoginUrl && !accountUrl}
      />
    </>
  );
}

export default Account;
