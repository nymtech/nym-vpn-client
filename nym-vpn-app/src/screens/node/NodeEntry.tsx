import { useEffect } from 'react';
import { NodesProvider, useMainState } from '../../contexts';
import { NodeHop } from '../../types';
import Node from './Node';

export type NodeEntryProps = {
  node: NodeHop;
};

function NodeEntry({ node }: NodeEntryProps) {
  const { vpnMode, fetchGateways } = useMainState();

  // refresh gateways cache in the background
  // (if needed like cache data is stale)
  useEffect(() => {
    if (vpnMode === 'mixnet') {
      fetchGateways(`mx-${node}`);
    } else {
      fetchGateways('wg');
    }
  }, [node, vpnMode, fetchGateways]);

  return (
    <NodesProvider nodeType={node}>
      <Node node={node} />
    </NodesProvider>
  );
}

export default NodeEntry;
