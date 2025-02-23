import { isCountry, isGateway } from '../../types';
import { Node } from './types';

export function isSelected(node: Node, selected: Node) {
  if (isGateway(node) && isGateway(selected)) {
    return selected.id === node.id;
  }
  if (isCountry(node) && isCountry(selected)) {
    return selected.code === node.code;
  }
  return false;
}

export function isSelectedNodeType(node: Node, selectedEntry: Node, selectedExit: Node): 'entry' | 'exit' | false {
  if (isSelected(node, selectedEntry)) return 'entry';
  if (isSelected(node, selectedExit)) return 'exit';
  return false;
}
