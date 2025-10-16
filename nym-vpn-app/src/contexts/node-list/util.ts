import { isCountry, isGateway } from '../../types';
import { Node, SelectedKind, UiCountry, UiGateway } from './types';

export function isSelected(node: Node, selected: Node) {
  if (isGateway(node) && isGateway(selected)) {
    return selected.id === node.id;
  }
  if (isCountry(node) && isCountry(selected)) {
    return selected.code === node.code;
  }
  return false;
}

export function isSelectedNodeType(
  node: Node,
  selectedEntry: Node,
  selectedExit: Node,
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
}: UiCountry | UiGateway): Node {
  return node;
}
