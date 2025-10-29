import { SelectableNode, SelectedNode, toSelectedNode } from '../../types';
import { SelectedKind, SelectedUiNode } from './types';

export function isSelected(node: SelectedNode, selected: SelectedNode) {
  if (node.type === 'gateway' && selected.type === 'gateway') {
    return selected.node.id === node.node.id;
  }
  if (node.type === 'country' && selected.type === 'country') {
    return selected.node.code === node.node.code;
  }
  if (node.type === 'region' && selected.type === 'region') {
    return selected.node.name === node.node.name;
  }
  return false;
}

export function isSelectedNodeType(
  node: SelectableNode,
  selectedEntry: SelectedNode,
  selectedExit: SelectedNode,
): SelectedKind {
  const selected = toSelectedNode(node);
  if (
    selected.type === 'country' &&
    isSelected(selected, selectedEntry) &&
    isSelected(selected, selectedExit)
  )
    return 'entry-and-exit';
  if (isSelected(selected, selectedEntry)) return 'entry';
  if (isSelected(selected, selectedExit)) return 'exit';
  return false;
}

export function uiNodeToSelectedNode(uiNode: SelectedUiNode): SelectedNode {
  switch (uiNode.nodeType) {
    case 'country':
      return {
        type: 'country',
        node: { code: uiNode.code, name: uiNode.name },
      };
    case 'region':
      return {
        type: 'region',
        node: { name: uiNode.name, country: uiNode.country },
      };
    case 'gateway': {
      return {
        type: 'gateway',
        node: {
          id: uiNode.id,
          name: uiNode.name,
          country: uiNode.country,
          city: uiNode.location.city,
          region: uiNode.location.region,
          asnType: uiNode.asn?.type,
        },
      };
    }
  }
}
