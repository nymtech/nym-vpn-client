import {
  SelectableNode,
  SelectedNode,
  isCountry,
  isGateway,
  isRegion,
  toSelectedNode,
} from '../../types';
import { SelectedKind, SelectedUiNode } from './types';

export function isSelected(node: SelectedNode, selected: SelectedNode) {
  if (isGateway(node) && isGateway(selected)) {
    return selected.gateway.id === node.gateway.id;
  }
  if (isCountry(node) && isCountry(selected)) {
    return selected.country.code === node.country.code;
  }
  if (isRegion(node) && isRegion(selected)) {
    return selected.region === node.region;
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
    isCountry(selected) &&
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
        country: { code: uiNode.code },
      };
    case 'region':
      return {
        region: uiNode.name,
      };
    case 'gateway': {
      return {
        gateway: { id: uiNode.id },
      };
    }
    case 'random':
      return 'random';
  }
}
