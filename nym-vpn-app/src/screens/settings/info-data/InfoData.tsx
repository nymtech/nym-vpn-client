import clsx from 'clsx';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { useMainState } from '../../../contexts';
import { routes } from '../../../router';
import { S_STATE } from '../../../static';
import { ButtonText } from '../../../ui';
import AccountData from './AccountData';

function InfoData() {
  const { version, daemonStatus, daemonVersion, networkEnv, account } =
    useMainState();
  const navigate = useNavigate();

  const { t } = useTranslation('settings');

  const copyToClipboard = async (text: string) => {
    try {
      await writeText(text);
    } catch (e) {
      console.error('failed to copy to clipboard', e);
    }
  };

  const InfoView = (
    <>
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
          <ButtonText onClick={() => copyToClipboard(networkEnv)} truncate>
            {networkEnv}
          </ButtonText>
        </div>
      )}
      {account && <AccountData />}
    </>
  );

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
          <ButtonText
            onClick={() => copyToClipboard(version || '')}
            onDoubleClick={() => S_STATE.devMode && navigate(routes.dev)}
            truncate
          >
            {version}
          </ButtonText>
        </div>
        {daemonStatus !== 'NotOk' && InfoView}
      </div>
    </>
  );
}

export default InfoData;
