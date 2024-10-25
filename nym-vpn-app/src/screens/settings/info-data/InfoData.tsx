import { useEffect, useState } from 'react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useMainState } from '../../../contexts';
import { DaemonInfo, NetworkEnv } from '../../../types';
import NetworkEnvSelect from './NetworkEnvSelect.tsx';
import { S_STATE } from '../../../static';

function InfoData() {
  const [daemonInfo, setDaemonInfo] = useState<DaemonInfo | undefined>();
  const [showEnvSelect, setShowEnvSelect] = useState(false);
  const { version, daemonStatus } = useMainState();

  const { t } = useTranslation('settings');

  useEffect(() => {
    const getInfo = async () => {
      try {
        const info = await invoke<DaemonInfo | undefined>('daemon_info');
        setDaemonInfo(info);
      } catch (e: unknown) {
        console.warn('failed to get daemon info', e);
        setDaemonInfo(undefined);
      }
    };

    getInfo();
  }, [daemonStatus]);

  return (
    <>
      <div
        className={clsx([
          'flex grow flex-col justify-end text-comet text-sm',
          'tracking-tight leading-tight mb-4 cursor-default',
        ])}
      >
        <p>{`${t('info.client-version')} ${version}`}</p>
        <p>
          {daemonInfo?.version &&
            `${t('info.daemon-version')} ${daemonInfo?.version}`}
        </p>
        <div
          onDoubleClick={() => {
            setShowEnvSelect(!showEnvSelect);
          }}
        >
          <p>
            {daemonInfo?.network &&
              daemonInfo?.network.length > 0 &&
              `${t('info.network-name')} ${daemonInfo?.network}`}
          </p>
        </div>
      </div>
      {S_STATE.networkEnvSelect &&
        daemonStatus === 'Ok' &&
        daemonInfo &&
        showEnvSelect && (
          <NetworkEnvSelect
            open={showEnvSelect}
            onClose={() => setShowEnvSelect(false)}
            current={daemonInfo.network as NetworkEnv}
          />
        )}
    </>
  );
}

export default InfoData;
