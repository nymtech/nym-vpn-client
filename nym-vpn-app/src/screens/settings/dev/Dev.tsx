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
    <div data-testid="dev-mixnet-data">
      <h3 className="text-lg mb-2" data-testid="dev-mixnet-title">
        Mixnet data
      </h3>
      <div
        className="bg-black/20 rounded-md flex flex-col gap-3 font-mono p-2 overflow-x-scroll"
        data-testid="dev-mixnet-details"
      >
        <div className="select-text cursor-text">
          {data.nymAddress && (
            <>
              {'nym address:'}
              <div data-testid="dev-mixnet-nym-address">
                {data.nymAddress?.nymAddress}
              </div>
            </>
          )}
          {data.exitIpr && (
            <>
              {'exit ipr:'}
              <div data-testid="dev-mixnet-exit-ipr">
                {data.exitIpr?.nymAddress}
              </div>
            </>
          )}
          <div data-testid="dev-mixnet-ipv4">{`ipv4: ${data.ipv4}`}</div>
          <div data-testid="dev-mixnet-ipv6">{`ipv6: ${data.ipv6}`}</div>
          <div data-testid="dev-mixnet-entry-ip">{`entry ip: ${data.entryIp}`}</div>
          <div data-testid="dev-mixnet-exit-ip">{`exit ip: ${data.exitIp}`}</div>
        </div>
      </div>
    </div>
  );

  const wgNode = (node: WgNode, nodeType: 'entry' | 'exit') => (
    <div
      className="select-text cursor-text"
      data-testid={`dev-wg-${nodeType}-node`}
    >
      <div
        data-testid={`dev-wg-${nodeType}-endpoint`}
      >{`endpoint: ${node.endpoint}`}</div>
      <div
        data-testid={`dev-wg-${nodeType}-ipv4`}
      >{`private ipv4: ${node.privateIpv4}`}</div>
      <div
        data-testid={`dev-wg-${nodeType}-ipv6`}
      >{`private ipv6: ${node.privateIpv6}`}</div>
      {'pub key:'}
      <div data-testid={`dev-wg-${nodeType}-pubkey`}>{node.publicKey}</div>
    </div>
  );

  const wgData = (data: WireguardData) => (
    <div data-testid="dev-wg-data">
      <h3 className="text-lg mb-2" data-testid="dev-wg-title">
        Wg data
      </h3>
      <div
        className="bg-black/20 rounded-md flex flex-col gap-3 font-mono p-2 overflow-x-scroll"
        data-testid="dev-wg-details"
      >
        entry:
        {wgNode(data.entry, 'entry')}
        exit:
        {wgNode(data.exit, 'exit')}
      </div>
    </div>
  );

  return (
    <PageAnim
      className="h-full flex flex-col py-6 gap-6 select-none cursor-default"
      data-testid="dev-page"
    >
      <SettingsMenuCard
        title={'CREDENTIALS_MODE'}
        onClick={() => credentialsModeChanged(!credentialsMode)}
        trailingComponent={
          <Switch
            checked={credentialsMode}
            onChange={credentialsModeChanged}
            data-testid="dev-credentials-switch"
          />
        }
        data-testid="dev-credentials-card"
      />
      {daemonStatus !== 'down' && networkEnv && (
        <NetworkEnvSelect current={networkEnv} />
      )}
      <div data-testid="dev-state-container">
        <h3 className="text-lg mb-2" data-testid="dev-state-title">
          State
        </h3>
        <div
          className="bg-black/20 rounded-md font-mono p-2 select-text cursor-text"
          data-testid="dev-state-value"
        >
          {state}
        </div>
      </div>
      {tunnel && (
        <div data-testid="dev-tunnel-container">
          <h3 className="text-lg mb-2" data-testid="dev-tunnel-title">
            Tunnel
          </h3>
          <div
            className="bg-black/20 rounded-md flex flex-col gap-3 font-mono p-2 overflow-x-scroll"
            data-testid="dev-tunnel-details"
          >
            <div>
              {'entry gw:'}
              <div
                className="select-text cursor-text"
                data-testid="dev-tunnel-entry-gw"
              >
                {tunnel.entryGwId}
              </div>
            </div>
            <div>
              {'exit gw:'}
              <div
                className="select-text cursor-text"
                data-testid="dev-tunnel-exit-gw"
              >
                {tunnel.exitGwId}
              </div>
            </div>
            {tunnel.connectedAt && (
              <div
                className="select-text cursor-text text-nowrap"
                data-testid="dev-tunnel-connected-at"
              >{`connectedAt: ${dayjs.unix(tunnel.connectedAt).format()}`}</div>
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
