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
} from '../../../ui';
import SettingsGroup from '../SettingsGroup';
import { CCache } from '../../../cache';
import { useInAppNotify, useMainState } from '../../../contexts';
import { routes } from '../../../router';
import { useDeepLink } from '../../../hooks';
import { getAccountColor, getAccountDescription } from './utils';

const IdsTimeToLive = 120; // sec

function Account() {
  const { t, i18n } = useTranslation('settings');
  const navigate = useNavigate();

  const { accountLinks, account, accountState, accountSyncing, daemonStatus } =
    useMainState();
  const needAPlan =
    account &&
    (accountState === 'no-subscription' ||
      accountState === 'bandwidth-exceeded');

  const [accountId, setAccountId] = useState<string | null>(null);
  const [deviceId, setDeviceId] = useState<string | null>(null);

  const { startListening } = useDeepLink();
  const { push } = useInAppNotify();

  const getAccountId = async () => {
    const accountId = await CCache.get<string>('cache-account-id');
    if (accountId) {
      setAccountId(accountId);
      return;
    }
    try {
      const accountId = await invoke<string>('get_account_id');
      setAccountId(accountId);
      CCache.set('cache-account-id', accountId, IdsTimeToLive);
    } catch {
      setAccountId(null);
    }
  };

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
    getAccountId();
    getDeviceId();
  }, []);

  const handleGoToAccount = async () => {
    const linkUrl = await invoke<string>('get_deep_link', {
      locale: i18n.language,
      kind: 'PrivyLink',
    });
    openUrl(linkUrl);

    try {
      const deeplinkUrl = await Promise.race([
        startListening(),
        new Promise<never>((_, reject) =>
          setTimeout(() => reject(new Error('Login timeout')), 300000),
        ),
      ]);

      await invoke('store_deeplink_account', {
        callbackUrl: deeplinkUrl,
      });
    } catch (error) {
      console.error('Account login error: ', error);
      if (error instanceof Error && error.message === 'Login timeout') {
        push({
          message: t('account-linking-timeout', { ns: 'notifications' }),
          type: 'error',
        });
      }
    }

    // if (accountLinks?.account) {
    //   openUrl(accountLinks.account);
    // } else if (accountLinks?.signIn) {
    //   openUrl(accountLinks.signIn);
    // }
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
            title: t('account.account-on-nym'),
            desc: (
              <span
                className={clsx(getAccountColor(accountSyncing, accountState))}
              >
                {getAccountDescription(t, accountSyncing, accountState) ??
                  t('account.account-link-social-description')}
              </span>
            ),
            leadingIcon: 'event_repeat',
            trailingIcon: 'open_in_new',
            onClick: handleGoToAccount,
          },
        ]}
      />

      <p className="text-sm text-iron dark:text-bombay">
        {t('account.account-linked')}
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
        <Button color="red" outline onClick={() => {}}>
          {t('account.logout')}
        </Button>
      </div>
    </PageAnim>
  );
}

export default Account;
