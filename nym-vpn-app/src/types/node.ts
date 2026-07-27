import {
  Country,
  Gateway,
  GatewaysByCountry,
  Region,
  SelectedNode,
} from './tauri';
import {
  SelectableNode,
  isCountry,
  isGateway,
  isRegion,
  toSelectedNode,
} from './util';

export type SelectedKind = 'entry-and-exit' | 'entry' | 'exit' | false;
export type GwSelectedKind = Exclude<SelectedKind, 'entry-and-exit'>;

export type SelectedUiNode =
  | UiCountry
  | UiRegion
  | UiGateway
  | { nodeType: 'random'; isSelected: SelectedKind };

export type UiCountry = Country & {
  nodeType: 'country';
  isSelected: SelectedKind;
  isFavorite: boolean;
};

export type UiGateway = Gateway & {
  nodeType: 'gateway';
  isSelected: GwSelectedKind;
  isFavorite: boolean;
};

export type UiRegion = Omit<Region, 'gateways'> & {
  nodeType: 'region';
  gateways: UiGateway[];
  isSelected: SelectedKind;
};

export type UiGatewaysByCountry = Omit<
  GatewaysByCountry,
  'gateways' | 'country' | 'quic' | 'regions'
> & {
  country: UiCountry;
  regions: UiRegion[];
  gateways: UiGateway[];
  i18n: string;
  isSelected: SelectedKind;
};

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

function isSelected(node: SelectedNode, selected: SelectedNode) {
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

export function uiNodeToSelectedNode(uiNode: SelectedUiNode): SelectedNode {
  switch (uiNode.nodeType) {
    case 'country':
      return { country: { code: uiNode.code } };
    case 'region':
      return { region: uiNode.name };
    case 'gateway':
      return { gateway: { id: uiNode.id } };
    case 'random':
      return 'random';
  }
}
