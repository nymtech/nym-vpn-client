import { Country, Gateway, GatewaysByCountry } from '../../types';

export type SelectedKind = 'entry' | 'exit' | false;

export type UiGateway = Gateway & { isSelected?: SelectedKind };

export type UiGatewaysByCountry = Omit<
  GatewaysByCountry,
  'gateways' | 'country'
> & {
  country: UiCountry;
  gateways: UiGateway[];
  i18n: string;
  isSelected: SelectedKind;
};

export type UiCountry = Country & { isSelected?: SelectedKind };

export type Node = Gateway | Country;
