import { Country, Gateway, GatewaysByCountry, Region } from '../../types';

export type SelectedKind = 'entry-and-exit' | 'entry' | 'exit' | false;
export type GwSelectedKind = Exclude<SelectedKind, 'entry-and-exit'>;

export type SelectedUiNode = UiCountry | UiRegion | UiGateway;
export type UiCountry = Country & { isSelected: SelectedKind };
export type UiGateway = Gateway & { isSelected: GwSelectedKind };

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

export type UiRegion = Omit<Region, 'gateways'> & {
  gateways: UiGateway[];
  // i18n: string;
  isSelected: SelectedKind;
};
