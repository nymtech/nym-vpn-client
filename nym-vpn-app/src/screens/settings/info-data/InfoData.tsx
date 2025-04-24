import clsx from 'clsx';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useMainState } from '../../../contexts';
import { useClipboard } from '../../../hooks';
import { routes } from '../../../router';
import { S_STATE } from '../../../static';
import { ButtonText } from '../../../ui';
import AccountData from './AccountData';

function InfoData() {
  const { version, daemonStatus, daemonVersion, networkEnv, account } =
    useMainState();
  const { copy } = useClipboard();

  const navigate = useNavigate();

  const { t } = useTranslation('settings');

  const InfoView = (
    <>
      {daemonVersion && (
        <div
          className={clsx('flex flex-row flex-nowrap gap-1')}
          data-testid="daemon-version-container"
        >
          <p className="text-nowrap" data-testid="daemon-version-label">
            {t('info.daemon-version')}
          </p>
          <ButtonText
            onClick={() => copy(daemonVersion)}
            truncate
            data-testid="daemon-version-value"
          >
            {daemonVersion}
          </ButtonText>
        </div>
      )}
      {networkEnv && networkEnv.length > 0 && (
        <div
          className={clsx('flex flex-row flex-nowrap gap-1')}
          data-testid="network-name-container"
        >
          <p className="text-nowrap" data-testid="network-name-label">
            {t('info.network-name')}
          </p>
          <ButtonText
            onClick={() => copy(networkEnv)}
            truncate
            data-testid="network-name-value"
          >
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
          'flex grow flex-col justify-end text-iron dark:text-iron text-sm',
          'tracking-tight leading-tight font-medium mb-4 cursor-default',
        ])}
        data-testid="info-data-container"
      >
        <div
          className={clsx('flex flex-row flex-nowrap gap-1')}
          data-testid="client-version-container"
        >
          <p className="text-nowrap" data-testid="client-version-label">
            {t('info.client-version')}
          </p>
          <ButtonText
            onClick={() => copy(version || '', !S_STATE.devMode)}
            onDoubleClick={() => S_STATE.devMode && navigate(routes.dev)}
            truncate
            data-testid="client-version-value"
          >
            {version}
          </ButtonText>
        </div>
        {daemonStatus !== 'down' && InfoView}
      </div>
    </>
  );
}

export default InfoData;
