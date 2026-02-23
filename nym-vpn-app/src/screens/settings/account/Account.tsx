import { useTranslation } from 'react-i18next';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useNavigate } from 'react-router';
import clsx from 'clsx';
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
import {
  useInAppNotify,
  useMainDispatch,
  useMainState,
} from '../../../contexts';
import { routes } from '../../../router';
import { useDeepLink, useLogout } from '../../../hooks';
import { StateDispatch, TAccountMode, TAccountSummary } from '../../../types';
import { getAccountColor, getAccountDescription } from './utils';

const IdsTimeToLive = 120; // sec

function Account() {
  const { t, i18n } = useTranslation('settings');
  const navigate = useNavigate();

  const { logout, loading } = useLogout();
  const {
    accountLinks,
    account,
    accountState,
    accountSyncing,
    daemonStatus,
    accountMode,
    accountSummary,
    backendFlags,
  } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const needAPlan =
    account &&
    (accountState === 'no-subscription' ||
      accountState === 'bandwidth-exceeded');

  const [isAccountLinking, setIsAccountLinking] = useState(false);
  const [deviceId, setDeviceId] = useState<string | null>(null);

  // Privy and linking logic
  const isLoggedWithPrivy = accountMode === 'privy';
  const isDifferentCanonical =
    accountSummary?.['account-addr'] !==
    accountSummary?.['canonical-account-addr'];
  const hasLinkedAuthMethod = accountSummary?.['auth-methods']?.some(
    (it) => it.label === 'Social login' || it.label === 'PassPhrase',
  );

  const isAccountLinked =
    isLoggedWithPrivy || isDifferentCanonical || hasLinkedAuthMethod;

  const { startListening } = useDeepLink();
  const { push } = useInAppNotify();

  const refreshAccount = async () => {
    try {
      const summary = await invoke<TAccountSummary>('get_account_summary');
      console.log('account summary', summary);
      dispatch({ type: 'set-account-summary', summary });
    } catch (err) {
      console.error('Failed to get account summary', err);
    }
    try {
      const mode = await invoke<TAccountMode>('get_account_mode');
      console.log('account mode', mode);
      dispatch({ type: 'set-account-mode', mode });
    } catch (err) {
      console.error('Failed to get account mode', err);
    }
  };

  useEffect(() => {
    refreshAccount();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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

  useEffect(() => {
    getDeviceId();
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

      const deeplinkUrl = await Promise.race([
        startListening(),
        new Promise<never>((_, reject) =>
          setTimeout(() => reject(new Error('Login timeout')), 300000),
        ),
      ]);

      await invoke('store_deeplink_account', {
        callbackUrl: deeplinkUrl,
      });
      await refreshAccount();
    } catch (error) {
      console.error('Account login error: ', error);
      if (error instanceof Error && error.message === 'Login timeout') {
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

  const handleManageSubscription = () => {
    if (accountLinks?.account) {
      openUrl(accountLinks.account);
    } else if (accountLinks?.signIn) {
      openUrl(accountLinks.signIn);
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

      <SettingsGroup
        settings={[
          {
            title: t('account.manage-subscriptoin'),
            desc: (
              <span
                className={clsx(getAccountColor(accountSyncing, accountState))}
              >
                {getAccountDescription(t, accountSyncing, accountState)}
              </span>
            ),
            leadingIcon: 'event_repeat',
            trailingIcon: 'open_in_new',
            onClick: handleManageSubscription,
          },
          ...(backendFlags.privy && isAccountLinked
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

      {backendFlags.privy && (
        <p className="text-sm text-iron dark:text-bombay">
          {isAccountLinked
            ? t('account.account-not-linked')
            : t('account.account-linked')}
        </p>
      )}

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
            value={accountSummary?.['canonical-account-addr'] ?? ''}
            label={accountSummary?.['canonical-account-addr'] ?? ''}
            loading={!accountSummary?.['canonical-account-addr']}
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

      {backendFlags.privy && (
        <p className="text-sm text-iron dark:text-bombay">
          {t('account.device-id-description')}
        </p>
      )}

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
