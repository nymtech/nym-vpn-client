import { useEffect, useState } from 'react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { useMainState } from '../../../contexts';
import { useClipboard } from '../../../hooks';
import { ButtonText } from '../../../ui';
import { getAccountId, getDeviceId } from '../../../utils';

function AccountData() {
  const [accountId, setAccountId] = useState<string | null>(null);
  const [deviceId, setDeviceId] = useState<string | null>(null);
  const { account } = useMainState();
  const { copy } = useClipboard();

  const { t } = useTranslation('settings');

  useEffect(() => {
    if (account) {
      getAccountId().then(setAccountId);
      getDeviceId().then(setDeviceId);
    }
  }, [account]);

  if (!account) {
    return null;
  }

  const truncateId = (id: string) => {
    if (id.length < 16) {
      return id;
    }
    return `${id.slice(0, 8)}…${id.slice(-8)}`;
  };

  return (
    <div className={clsx('mt-3')} data-testid="account-data-container">
      {accountId && (
        <div
          className={clsx('flex flex-row flex-nowrap gap-1')}
          data-testid="account-id-container"
        >
          <p className="text-nowrap" data-testid="account-id-label">
            {t('info.account-id')}
          </p>
          <ButtonText
            onClick={() => copy(accountId)}
            truncate
            data-testid="account-id-value"
          >
            {truncateId(accountId)}
          </ButtonText>
        </div>
      )}
      {deviceId && (
        <div
          className={clsx('flex flex-row flex-nowrap gap-1')}
          data-testid="device-id-container"
        >
          <p className="text-nowrap" data-testid="device-id-label">
            {t('info.device-id')}
          </p>
          <ButtonText
            onClick={() => copy(deviceId)}
            truncate
            data-testid="device-id-value"
          >
            {truncateId(deviceId)}
          </ButtonText>
        </div>
      )}
    </div>
  );
}

export default AccountData;
