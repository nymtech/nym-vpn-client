import { Country, Gateway, GatewaysByCountry } from '../../types';

export type UiGateway = Gateway & { isSelected?: 'entry' | 'exit' | false };

export type UiGatewaysByCountry = Omit<GatewaysByCountry, 'gateways'> & {
  gateways: UiGateway[];
  i18n: string;
  isSelected?: 'entry' | 'exit' | false;
};

export type Node = Gateway | Country;
