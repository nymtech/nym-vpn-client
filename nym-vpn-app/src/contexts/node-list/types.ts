import { Country, Gateway, GatewaysByCountry, Region } from '../../types';

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
};
export type UiGateway = Gateway & {
  nodeType: 'gateway';
  isSelected: GwSelectedKind;
};
export type UiRegion = Omit<Region, 'gateways'> & {
  nodeType: 'region';
  gateways: UiGateway[];
  // i18n: string;
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
