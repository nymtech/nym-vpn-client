import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { PageAnim, SettingsMenuCard, Switch } from '../../../ui';
import { useMainState } from '../../../contexts';
import NetworkEnvSelect from './NetworkEnvSelect';

function Dev() {
  const [credentialsMode, setCredentialsMode] = useState(false);

  const { daemonStatus, networkEnv } = useMainState();

  useEffect(() => {
    const getCredentialsMode = async () => {
      const enabled = await invoke<boolean>('get_credentials_mode');
      console.log('credentials mode:', enabled);
      setCredentialsMode(enabled);
    };
    getCredentialsMode();
  }, []);

  const credentialsModeChanged = (enabled: boolean) => {
    invoke('set_credentials_mode', { enabled }).then(() => {
      setCredentialsMode(enabled);
    });
  };

  return (
    <PageAnim className="h-full flex flex-col py-6 gap-6 select-none cursor-default">
      <SettingsMenuCard
        title={'CREDENTIALS_MODE'}
        onClick={() => credentialsModeChanged(!credentialsMode)}
        trailingComponent={
          <Switch checked={credentialsMode} onChange={credentialsModeChanged} />
        }
      />
      {daemonStatus !== 'NotOk' && networkEnv && (
        <NetworkEnvSelect current={networkEnv} />
      )}
    </PageAnim>
  );
}

export default Dev;
