import {
  Region,
  SelectedNode,
  isCountry,
  isGateway,
  isRegion,
} from '../../types';
import { SelectedKind, SelectedUiNode, UiRegion } from './types';

export function regionToSelectedNode(region: Region | UiRegion): SelectedNode {
  return {
    name: region.name,
    country: region.country,
  };
}

export function isSelected(node: SelectedNode, selected: SelectedNode) {
  if (isGateway(node) && isGateway(selected)) {
    return selected.id === node.id;
  }
  if (isCountry(node) && isCountry(selected)) {
    return selected.code === node.code;
  }
  if (isRegion(node) && isRegion(selected)) {
    return selected.name === node.name;
  }
  return false;
}

export function isSelectedNodeType(
  node: SelectedNode,
  selectedEntry: SelectedNode,
  selectedExit: SelectedNode,
): SelectedKind {
  if (
    isCountry(node) &&
    isSelected(node, selectedEntry) &&
    isSelected(node, selectedExit)
  )
    return 'entry-and-exit';
  if (isSelected(node, selectedEntry)) return 'entry';
  if (isSelected(node, selectedExit)) return 'exit';
  return false;
}

export function uiNodeToRaw({
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  isSelected,
  ...node
}: SelectedUiNode): SelectedNode {
  // TODO need to be fixed
  return node;
}
