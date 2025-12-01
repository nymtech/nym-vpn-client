import { memo } from 'react';
import { dequal } from 'dequal';
import { Accordion } from '@base-ui-components/react';
import {
  Focused,
  SelectedKind,
  SelectedUiNode,
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
  UiRegion,
  useMainState,
  useNodeListState,
} from '../../../contexts';
import { NodeHop, VpnMode } from '../../../types';
import { NodeItem } from './NodeItem';

export type NodeListProps = {
  nodes: UiGatewaysByCountry[];
  gateways: UiGateway[];
  onSelect: (node: SelectedUiNode) => void;
  onNodeDetails: (node: UiGateway) => void;
  hop: NodeHop;
  vpnMode: VpnMode;
  expanded: string[];
  focused: Focused | null;
};

const NodeList = memo(function NodeList({
  nodes,
  onSelect,
  hop,
  vpnMode,
  onNodeDetails,
  expanded,
}: NodeListProps) {
  const { backendFlags, quic } = useMainState();
  const { setExpanded } = useNodeListState();

  const quicFilter =
    vpnMode === 'wg' && hop === 'entry' && backendFlags.quic && quic;

  const handleLocationSelect = (
    location: UiCountry | UiRegion,
    isSelected: SelectedKind,
    gwCount: number,
  ) => {
    if (isSelected && isSelected !== hop && gwCount <= 1) {
      // don't allow selecting a country if it has only one gateway,
      // and it's already selected by the other hop
      return;
    }
    if (isSelected !== hop && isSelected !== 'entry-and-exit') {
      onSelect(location);
    }
  };

  const onValueChange = (value: string[]) => {
    setExpanded(hop, value);
  };

  return (
    <div className="mr-0">
      <Accordion.Root
        className="w-full flex flex-col gap-3"
        data-testid="node-list-accordion"
        value={expanded}
        onValueChange={onValueChange}
        multiple
      >
        {nodes.map((node) => (
          <Accordion.Item
            key={node.country.code}
            value={node.country.code}
            render={() => (
              <NodeItem
                key={node.country.code}
                node={node}
                hop={hop}
                vpnMode={vpnMode}
                quicFilter={quicFilter}
                handleLocationSelect={handleLocationSelect}
                onGatewaySelect={onSelect}
                onNodeDetails={onNodeDetails}
              />
            )}
          ></Accordion.Item>
        ))}
      </Accordion.Root>
    </div>
  );
}, arePropsEqual);

export default NodeList;

function arePropsEqual(
  oldProps: NodeListProps,
  newProps: NodeListProps,
): boolean {
  if (oldProps.hop !== newProps.hop) return false;
  if (oldProps.vpnMode !== newProps.vpnMode) return false;
  if (oldProps.gateways.length !== newProps.gateways.length) return false;
  if (oldProps.nodes.length !== newProps.nodes.length) return false;
  if (!dequal(oldProps.expanded, newProps.expanded)) return false;
  if (!dequal(oldProps.focused, newProps.focused)) return false;
  if (!dequal(oldProps.gateways, newProps.gateways)) return false;
  if (!dequal(oldProps.nodes, newProps.nodes)) return false;
  return true;
}
