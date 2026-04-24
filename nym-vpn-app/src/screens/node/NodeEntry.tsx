import { useEffect } from 'react';
import * as _ from 'lodash-es';
import { useShallow } from 'zustand/react/shallow';
import { NodeListProvider, useFetchGateways } from '../../contexts';
import { useAppStore } from '../../store';
import { NodeHop, VpnMode } from '../../types';
import Node from './Node';

export type NodeEntryProps = {
  node: NodeHop;
};

function NodeEntry({ node }: NodeEntryProps) {
  const { daemonStatus, vpnMode } = useAppStore(
    useShallow((s) => ({
      daemonStatus: s.daemonStatus,
      vpnMode: s.vpnMode,
    })),
  );
  const fetchGateways = useFetchGateways();

  const refresh = _.throttle(
    async (mode: VpnMode) => {
      if (mode === 'mixnet') {
        await fetchGateways(`mx-${node}`);
      } else {
        await fetchGateways('wg');
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
    <NodeListProvider hop={node}>
      <Node node={node} />
    </NodeListProvider>
  );
}

export default NodeEntry;
