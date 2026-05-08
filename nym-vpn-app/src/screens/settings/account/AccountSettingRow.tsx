import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useEffect } from 'react';
import { dispatch, useMainState } from '../../../store';
import { Button, ButtonNew, MsIcon } from '../../../ui';
import { routes } from '../../../router';
import SettingsGroup from '../SettingsGroup';
import { AccountDescription } from './AccountDescription';

function AccountSettingRow() {
  const { daemonStatus, account, accountState, accountSyncing } =
    useMainState();

  const navigate = useNavigate();
  const { t } = useTranslation('settings');
  const needAPlan = account && accountState === 'no-subscription';

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
  }, [daemonStatus]);

  if (!account) {
    return (
      <ButtonNew
        onClick={() => navigate(routes.onboarding)}
        disabled={daemonStatus === 'down'}
      >
        {t('account.get-started')}
      </ButtonNew>
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
            leadingIcon: 'person',
            onClick: () => navigate(routes.accountSettings),
            trailing: (
              <MsIcon icon="chevron_right" className="dark:text-white" />
            ),
          },
        ]}
      />
    </>
  );
}

export default AccountSettingRow;
