import dayjs from 'dayjs';
import { PageAnim } from '../../../ui';
import { useMainState } from '../../../store';
import {
  MixnetData,
  WgNode,
  WireguardData,
  isMixnetData,
  isWireguardData,
} from '../../../types';
import NetworkEnvSelect from './NetworkEnvSelect';

function Dev() {
  const { daemonStatus, networkEnv, tunnel, state } = useMainState();

  const mixnetData = (data: MixnetData) => (
    <div data-testid="dev-mixnet-data">
      <h3 className="mb-2 text-lg" data-testid="dev-mixnet-title">
        Mixnet data
      </h3>
      <div
        className="flex flex-col gap-3 overflow-x-scroll rounded-md bg-black/20 p-2 font-mono"
        data-testid="dev-mixnet-details"
      >
        <div className="cursor-text select-text">
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
      className="cursor-text select-text"
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
      <h3 className="mb-2 text-lg" data-testid="dev-wg-title">
        Wg data
      </h3>
      <div
        className="flex flex-col gap-3 overflow-x-scroll rounded-md bg-black/20 p-2 font-mono"
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
      className="flex h-full cursor-default flex-col gap-6 py-6 select-none"
      data-testid="dev-page"
    >
      {daemonStatus !== 'down' && networkEnv && (
        <NetworkEnvSelect current={networkEnv} />
      )}
      <div data-testid="dev-state-container">
        <h3 className="mb-2 text-lg" data-testid="dev-state-title">
          State
        </h3>
        <div
          className="cursor-text rounded-md bg-black/20 p-2 font-mono select-text"
          data-testid="dev-state-value"
        >
          {state}
        </div>
      </div>
      {tunnel && (
        <div data-testid="dev-tunnel-container">
          <h3 className="mb-2 text-lg" data-testid="dev-tunnel-title">
            Tunnel
          </h3>
          <div
            className="flex flex-col gap-3 overflow-x-scroll rounded-md bg-black/20 p-2 font-mono"
            data-testid="dev-tunnel-details"
          >
            <div>
              {'entry gw:'}
              <div
                className="cursor-text select-text"
                data-testid="dev-tunnel-entry-gw"
              >
                {tunnel.entryGwId}
              </div>
            </div>
            <div>
              {'exit gw:'}
              <div
                className="cursor-text select-text"
                data-testid="dev-tunnel-exit-gw"
              >
                {tunnel.exitGwId}
              </div>
            </div>
            {tunnel.connectedAt && (
              <div
                className="cursor-text text-nowrap select-text"
                data-testid="dev-tunnel-connected-at"
              >{`connectedAt: ${dayjs.unix(tunnel.connectedAt as unknown as number).format()}`}</div>
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
