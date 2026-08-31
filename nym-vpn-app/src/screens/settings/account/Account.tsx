import { Trans, useTranslation } from 'react-i18next';
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
import { useAutologin } from '../../../contexts';
import { useMainState } from '../../../store';
import { routes } from '../../../router';
import {
  useDeepLink,
  useLogout,
  useRefreshAccountSummary,
  useToast,
} from '../../../hooks';
import { DeeplinkTimeout } from '../../../errors';
import { ContactSupportUrl } from '../../../constants';
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
  const { add } = useToast();
  const { refresh, refreshing } = useRefreshAccountSummary();

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

  // Force-refresh account state/summary each time the account view opens.
  useEffect(() => {
    if (account) {
      refresh().catch((error: unknown) => {
        console.error('Failed to refresh account state on mount: ', error);
      });
    }
  }, [account, refresh]);

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
        add({
          title: t('account-linking-timeout', { ns: 'notifications' }),
          type: 'error',
        });
      } else {
        add({
          title: t('account-linking-error', { ns: 'notifications' }),
          type: 'error',
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
    <PageAnim className="mt-2 flex h-full flex-col gap-6 pb-2 select-none">
      {needAPlan && (
        <Button
          onClick={() => navigate(routes.selectPlan)}
          disabled={daemonStatus === 'down' || accountSyncing}
          variant="primary"
        >
          {t('account.choose-plan')}
        </Button>
      )}

      <AccountStatus refresh={refresh} refreshing={refreshing} />

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
        ]}
      />

      <div className="flex flex-row items-center gap-2">
        <div className="bg-status-warning h-4 w-4 rounded-full"></div>
        <span className="text-text-secondary text-sm">
          {accountSummary?.isLinked ? (
            t('account.account-linked')
          ) : (
            <Trans
              i18nKey="account.account-not-linked"
              ns="settings"
              components={{
                button: (
                  <button
                    className="hover:text-shadow-text-primary underline dark:hover:text-white"
                    onClick={handleAccountLink}
                  />
                ),
              }}
            />
          )}
        </span>
        {isAccountLinking && <Spinner className="h-4! w-4! border-2!" />}
      </div>

      <CardNew>
        <CardNewHeader>
          <div className="flex flex-row items-center gap-2">
            <MsIcon icon="numbers" className="text-text-secondary" />
            <p className="text-text-primary truncate text-left text-base select-none">
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

      <span className="text-text-secondary text-sm">
        <Trans
          ns="settings"
          i18nKey="account.account-id-description"
          components={{
            support: (
              <button
                type="button"
                className="hover:text-shadow-text-primary underline dark:hover:text-white"
                onClick={() => openUrl(ContactSupportUrl)}
              />
            ),
          }}
        />
      </span>

      <CardNew>
        <CardNewHeader>
          <div className="flex flex-row items-center gap-2">
            <MsIcon icon="monitor" className="text-text-secondary" />
            <p className="text-text-primary truncate text-left text-base select-none">
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

      <div className="flex flex-col gap-2">
        <Button
          variant="destructive-outlined"
          onClick={() => logout()}
          disabled={loading}
          loading={loading}
        >
          {t('account.logout')}
        </Button>
      </div>
    </PageAnim>
  );
}

export default Account;
