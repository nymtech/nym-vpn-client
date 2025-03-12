import { useEffect } from 'react';
import * as _ from 'lodash-es';
import { NodesProvider, useGateways, useMainState } from '../../contexts';
import { NodeHop, VpnMode } from '../../types';
import Node from './Node';

export type NodeEntryProps = {
  node: NodeHop;
};

function NodeEntry({ node }: NodeEntryProps) {
  const { daemonStatus, vpnMode } = useMainState();
  const { fetch } = useGateways();

  const refresh = _.throttle(
    async (mode: VpnMode) => {
      if (mode === 'mixnet') {
        await fetch(`mx-${node}`);
      } else {
        await fetch('wg');
      }
    },
    5000,
    {
      trailing: false,
    },
  );

  // refresh gateways in the background
  // (only if needed ie. no cache data or cache is stale)
  useEffect(() => {
    if (daemonStatus === 'down') {
      return;
    }
    // during development useEffect is fired twice
    // to avoid unnecessary fetch calls, throttle the refresh
    // see https://react.dev/learn/synchronizing-with-effects#how-to-handle-the-effect-firing-twice-in-development
    refresh(vpnMode);
    // ⚠ do not include `refresh` in the dependencies array
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [node, vpnMode, daemonStatus]);

  return (
    <NodesProvider nodeType={node}>
      <Node node={node} />
    </NodesProvider>
  );
}

export default NodeEntry;
