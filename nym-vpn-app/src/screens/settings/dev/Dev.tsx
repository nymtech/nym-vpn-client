import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import dayjs from 'dayjs';
import { PageAnim, SettingsMenuCard, Switch } from '../../../ui';
import { useMainState } from '../../../contexts';
import {
  MixnetData,
  WgNode,
  WireguardData,
  isMixnetData,
  isWireguardData,
} from '../../../types';
import NetworkEnvSelect from './NetworkEnvSelect';

function Dev() {
  const [credentialsMode, setCredentialsMode] = useState(false);

  const { daemonStatus, networkEnv, tunnel, state } = useMainState();

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

  const mixnetData = (data: MixnetData) => (
    <div>
      <h3 className="text-lg mb-2">Mixnet data</h3>
      <div className="bg-black/20 rounded-md flex flex-col gap-3 font-mono p-2 overflow-x-scroll">
        <div className="select-text cursor-text">
          {data.nymAddress && (
            <>
              {'nym address:'}
              <div>{data.nymAddress?.nymAddress}</div>
            </>
          )}
          {data.exitIpr && (
            <>
              {'exit ipr:'}
              <div>{data.exitIpr?.nymAddress}</div>
            </>
          )}
          <div>{`ipv4: ${data.ipv4}`}</div>
          <div>{`ipv6: ${data.ipv6}`}</div>
          <div>{`entry ip: ${data.entryIp}`}</div>
          <div>{`exit ip: ${data.exitIp}`}</div>
        </div>
      </div>
    </div>
  );

  const wgNode = (node: WgNode) => (
    <div className="select-text cursor-text">
      <div>{`endpoint: ${node.endpoint}`}</div>
      <div>{`private ipv4: ${node.privateIpv4}`}</div>
      <div>{`private ipv6: ${node.privateIpv6}`}</div>
      {'pub key:'}
      <div>{node.publicKey}</div>
    </div>
  );

  const wgData = (data: WireguardData) => (
    <div>
      <h3 className="text-lg mb-2">Wg data</h3>
      <div className="bg-black/20 rounded-md flex flex-col gap-3 font-mono p-2 overflow-x-scroll">
        entry:
        {wgNode(data.entry)}
        exit:
        {wgNode(data.exit)}
      </div>
    </div>
  );

  return (
    <PageAnim className="h-full flex flex-col py-6 gap-6 select-none cursor-default">
      <SettingsMenuCard
        title={'CREDENTIALS_MODE'}
        onClick={() => credentialsModeChanged(!credentialsMode)}
        trailingComponent={
          <Switch checked={credentialsMode} onChange={credentialsModeChanged} />
        }
      />
      {daemonStatus !== 'down' && networkEnv && (
        <NetworkEnvSelect current={networkEnv} />
      )}
      <div>
        <h3 className="text-lg mb-2">State</h3>
        <div className="bg-black/20 rounded-md font-mono p-2 select-text cursor-text">
          {state}
        </div>
      </div>
      {tunnel && (
        <div>
          <h3 className="text-lg mb-2">Tunnel</h3>
          <div className="bg-black/20 rounded-md flex flex-col gap-3 font-mono p-2 overflow-x-scroll">
            <div>
              {'entry gw:'}
              <div className="select-text cursor-text">{tunnel.entryGwId}</div>
            </div>
            <div>
              {'exit gw:'}
              <div className="select-text cursor-text">{tunnel.exitGwId}</div>
            </div>
            {tunnel.connectedAt && (
              <div className="select-text cursor-text text-nowrap">{`connectedAt: ${dayjs.unix(tunnel.connectedAt).format()}`}</div>
            )}
          </div>
        </div>
      )}
      {tunnel && isMixnetData(tunnel.data) && mixnetData(tunnel.data)}
      {tunnel && isWireguardData(tunnel.data) && wgData(tunnel.data)}
    </PageAnim>
  );
}

export default Dev;
