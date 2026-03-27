import { useTranslation } from 'react-i18next';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useNavigate } from 'react-router';
import {
  Button,
  CardNew,
  CardNewBody,
  CardNewCopyableRow,
  CardNewHeader,
  MsIcon,
  PageAnim,
  Spinner,
} from '../../../ui';
import SettingsGroup from '../SettingsGroup';
import { CCache } from '../../../cache';
import { useAutologin, useInAppNotify, useMainState } from '../../../contexts';
import { routes } from '../../../router';
import { useDeepLink, useLogout } from '../../../hooks';
import { DeeplinkTimeout } from '../../../errors';
import { AccountStatus } from './account-status';
import { AccountDescription } from './AccountDescription';

const IdsTimeToLive = 120; // sec

function Account() {
  const { t, i18n } = useTranslation('settings');
  const navigate = useNavigate();

  const { logout, loading } = useLogout();
  const {
    account,
    accountState,
    accountSyncing,
    daemonStatus,
    accountSummary,
  } = useMainState();
  const [autologinLoading, setAutologinLoading] = useState(false);
  const { autologin } = useAutologin();
  const needAPlan = account && accountState === 'no-subscription';

  const [isAccountLinking, setIsAccountLinking] = useState(false);
  const [deviceId, setDeviceId] = useState<string | null>(null);
  const [accountId, setAccountId] = useState<string | null>(null);

  const { startListening } = useDeepLink();
  const { push } = useInAppNotify();

  const getDeviceId = async () => {
    const deviceId = await CCache.get<string>('cache-device-id');
    if (deviceId) {
      setDeviceId(deviceId);
      return;
    }
    try {
      const deviceId = await invoke<string>('get_device_id');
      setDeviceId(deviceId);
      CCache.set('cache-device-id', deviceId, IdsTimeToLive);
    } catch {
      setDeviceId(null);
    }
  };

  const getAccountId = async () => {
    const accountId = await CCache.get<string>('cache-account-id');
    if (accountId) {
      setAccountId(accountId);
      return;
    }
    try {
      const accountId = await invoke<string>('get_canonical_account_id');
      setAccountId(accountId);
      CCache.set('cache-account-id', accountId, IdsTimeToLive);
    } catch {
      setAccountId(null);
    }
  };

  useEffect(() => {
    getDeviceId();
    getAccountId();
  }, []);

  // When logged out, navigate to settings
  useEffect(() => {
    if (!account) navigate(routes.settings, { replace: true });
  }, [account, navigate]);

  const handleAccountLink = async () => {
    setIsAccountLinking(true);

    try {
      const linkUrl = await invoke<string>('get_deep_link', {
        locale: i18n.language,
        kind: 'PrivyLink',
      });
      openUrl(linkUrl);

      const deeplinkUrl = await startListening(300000);

      await invoke('store_deeplink_account', {
        callbackUrl: deeplinkUrl,
      });
    } catch (error) {
      console.error('Account login error: ', error);
      if (error instanceof DeeplinkTimeout) {
        push({
          message: t('account-linking-timeout', { ns: 'notifications' }),
          type: 'error',
          duration: 3000,
          close: true,
        });
      } else {
        push({
          message: t('account-linking-error', { ns: 'notifications' }),
          type: 'error',
          duration: 3000,
          close: true,
        });
      }
    } finally {
      setIsAccountLinking(false);
    }
  };

  const handleManageSubscription = async () => {
    setAutologinLoading(true);

    try {
      await autologin('autologinView');

      // don't block. User may or may not do changes on the website.
      void (async () => {
        await startListening(600000);

        await invoke<void>('handle_subscription_payment');
      })();
    } finally {
      setAutologinLoading(false);
    }
  };

  return (
    <PageAnim className="h-full flex flex-col mt-2 pb-2 gap-6 select-none">
      {needAPlan && (
        <Button
          onClick={() => navigate(routes.selectPlan)}
          disabled={daemonStatus === 'down' || accountSyncing}
        >
          {t('account.choose-plan')}
        </Button>
      )}

      <AccountStatus />

      <SettingsGroup
        settings={[
          {
            title: t('account.manage-subscriptoin'),
            desc: <AccountDescription />,
            leadingIcon: 'event_repeat',
            trailingIcon: autologinLoading ? undefined : 'open_in_new',
            trailing: autologinLoading ? <Spinner /> : undefined,
            onClick: handleManageSubscription,
          },
          ...(!accountSummary?.['is-linked']
            ? [
                {
                  title: t('account.account-on-nym'),
                  desc: t('account.account-link-social-description'),
                  leadingIcon: isAccountLinking ? undefined : 'person',
                  leadingComponent: isAccountLinking ? <Spinner /> : undefined,
                  trailingIcon: 'open_in_new',
                  onClick: handleAccountLink,
                },
              ]
            : []),
        ]}
      />

      <p className="text-sm text-iron dark:text-bombay">
        {accountSummary?.['is-linked']
          ? t('account.account-linked')
          : t('account.account-not-linked')}
      </p>

      <CardNew>
        <CardNewHeader>
          <div className="flex flex-row items-center gap-2">
            <MsIcon icon="numbers" className="text-iron dark:text-bombay" />
            <p className="text-left truncate text-base text-baltic-sea dark:text-white select-none">
              {t('account.account-id')}
            </p>
          </div>
        </CardNewHeader>
        <CardNewBody className="pb-5">
          <CardNewCopyableRow
            // Displaying canonical account address, as this is NYM's default account address
            value={accountId ?? ''}
            label={accountId ?? ''}
            loading={!accountId}
          />
        </CardNewBody>
      </CardNew>

      <p className="text-sm text-iron dark:text-bombay">
        {t('account.account-id-description')}
      </p>

      <CardNew>
        <CardNewHeader>
          <div className="flex flex-row items-center gap-2">
            <MsIcon icon="devices" className="text-iron dark:text-bombay" />
            <p className="text-left truncate text-base text-baltic-sea dark:text-white select-none">
              {t('account.device-id')}
            </p>
          </div>
        </CardNewHeader>
        <CardNewBody className="pb-5">
          <CardNewCopyableRow
            value={deviceId ?? ''}
            label={deviceId ?? ''}
            loading={!deviceId}
          />
        </CardNewBody>
      </CardNew>

      <p className="text-sm text-iron dark:text-bombay">
        {t('account.device-id-description')}
      </p>

      <div className="flex flex-col gap-2">
        <Button
          color="red"
          outline
          onClick={() => logout()}
          disabled={loading}
          spinner={loading}
        >
          {t('account.logout')}
        </Button>
      </div>
    </PageAnim>
  );
}

export default Account;
