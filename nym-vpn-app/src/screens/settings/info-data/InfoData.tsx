import { useState } from 'react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { useMainState } from '../../../contexts';
import NetworkEnvSelect from './NetworkEnvSelect';
import { S_STATE } from '../../../static';

function InfoData() {
  const [showEnvSelect, setShowEnvSelect] = useState(false);
  const { version, daemonStatus, daemonVersion, networkEnv } = useMainState();

  const { t } = useTranslation('settings');

  return (
    <>
      <div
        className={clsx([
          'flex grow flex-col justify-end text-comet text-sm',
          'tracking-tight leading-tight mb-4 cursor-default',
        ])}
      >
        <p>{`${t('info.client-version')} ${version}`}</p>
        <p>{daemonVersion && `${t('info.daemon-version')} ${daemonVersion}`}</p>
        <div
          onDoubleClick={() => {
            setShowEnvSelect(!showEnvSelect);
          }}
        >
          <p>
            {networkEnv &&
              networkEnv.length > 0 &&
              `${t('info.network-name')} ${networkEnv}`}
          </p>
        </div>
      </div>
      {S_STATE.networkEnvSelect &&
        daemonStatus === 'Ok' &&
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
