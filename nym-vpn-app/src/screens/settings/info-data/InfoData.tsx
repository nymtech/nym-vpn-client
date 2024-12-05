import { useState } from 'react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { useMainState } from '../../../contexts';
import NetworkEnvSelect from './NetworkEnvSelect';
import { S_STATE } from '../../../static';
import { ButtonText } from '../../../ui';
import AccountData from './AccountData';

function InfoData() {
  const [showEnvSelect, setShowEnvSelect] = useState(false);
  const { version, daemonStatus, daemonVersion, networkEnv, account } =
    useMainState();

  const { t } = useTranslation('settings');

  const copyToClipboard = async (text: string) => {
    try {
      await writeText(text);
    } catch (e) {
      console.error('failed to copy to clipboard', e);
    }
  };

  return (
    <>
      <div
        className={clsx([
          'select-none',
          'flex grow flex-col justify-end text-comet/80 text-sm',
          'tracking-tight leading-tight font-semibold mb-4 cursor-default',
        ])}
      >
        <div className={clsx('flex flex-row flex-nowrap gap-1')}>
          <p className="text-nowrap">{t('info.client-version')}</p>
          <ButtonText onClick={() => copyToClipboard(version || '')} truncate>
            {version}
          </ButtonText>
        </div>
        {daemonVersion && (
          <div className={clsx('flex flex-row flex-nowrap gap-1')}>
            <p className="text-nowrap">{t('info.daemon-version')}</p>
            <ButtonText onClick={() => copyToClipboard(daemonVersion)} truncate>
              {daemonVersion}
            </ButtonText>
          </div>
        )}
        {networkEnv && networkEnv.length > 0 && (
          <div className={clsx('flex flex-row flex-nowrap gap-1')}>
            <p className="text-nowrap">{t('info.network-name')}</p>
            <ButtonText
              onClick={() => copyToClipboard(networkEnv)}
              onDoubleClick={() => setShowEnvSelect(!showEnvSelect)}
              truncate
            >
              {networkEnv}
            </ButtonText>
          </div>
        )}
        {account && <AccountData />}
      </div>
      {S_STATE.networkEnvSelect &&
        daemonStatus !== 'NotOk' &&
        networkEnv &&
        showEnvSelect && (
          <NetworkEnvSelect
            open={showEnvSelect}
            onClose={() => setShowEnvSelect(false)}
            current={networkEnv}
          />
        )}
    </>
  );
}

export default InfoData;
