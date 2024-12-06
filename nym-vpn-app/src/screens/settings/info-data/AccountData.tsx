import { useEffect, useState } from 'react';
import clsx from 'clsx';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useMainState } from '../../../contexts';
import { MCache } from '../../../cache';
import { ButtonText } from '../../../ui';

const IdsTimeToLive = 120; // sec

function AccountData() {
  const [accountId, setAccountId] = useState<string | null>(null);
  const [deviceId, setDeviceId] = useState<string | null>(null);
  const { account } = useMainState();

  const { t } = useTranslation('settings');

  const getAccountId = async () => {
    const id = MCache.get<string>('account-id');
    if (id) {
      setAccountId(id);
      return;
    }
    try {
      const id = await invoke<string | null>('get_account_id');
      setAccountId(id);
      MCache.set('account-id', id, IdsTimeToLive);
    } catch (e) {
      console.warn('failed to get account id', e);
      setAccountId(null);
    }
  };

  const getDeviceId = async () => {
    const id = MCache.get<string>('device-id');
    if (id) {
      setDeviceId(id);
      return;
    }
    try {
      const id = await invoke<string | null>('get_device_id');
      setDeviceId(id);
      MCache.set('device-id', id, IdsTimeToLive);
    } catch (e) {
      console.warn('failed to get device id', e);
      setDeviceId(null);
    }
  };

  useEffect(() => {
    if (account) {
      getAccountId();
      getDeviceId();
    }
  }, [account]);

  const copyToClipboard = async (text: string) => {
    try {
      await writeText(text);
    } catch (e) {
      console.error('failed to copy to clipboard', e);
    }
  };

  if (!account) {
    return null;
  }

  return (
    <div className={clsx('mt-3')}>
      {accountId && (
        <div className={clsx('flex flex-row flex-nowrap gap-1')}>
          <p className="text-nowrap">{t('info.account-id')}</p>
          <ButtonText onClick={() => copyToClipboard(accountId)} truncate>
            {accountId}
          </ButtonText>
        </div>
      )}
      {deviceId && (
        <div className={clsx('flex flex-row flex-nowrap gap-1')}>
          <p className="text-nowrap">{t('info.device-id')}</p>
          <ButtonText onClick={() => copyToClipboard(deviceId)} truncate>
            {deviceId}
          </ButtonText>
        </div>
      )}
    </div>
  );
}

export default AccountData;
