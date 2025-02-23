import { NodesProvider } from '../../contexts';
import { NodeHop } from '../../types';
import Node from './Node';

export type NodeEntryProps = {
  node: NodeHop;
};

function NodeEntry({ node }: NodeEntryProps) {
  return (
    <NodesProvider nodeType={node}>
      <Node node={node} />
    </NodesProvider>
  );
}

export default NodeEntry;
