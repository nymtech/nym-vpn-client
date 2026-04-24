import {
  AppError,
  Gateway,
  GatewayType,
  GatewaysByCountry,
} from '../../../types';

export type GatewaysSlice = GatewaysState & {
  fetchGateways: (nodeType: GatewayType) => Promise<void>;
  lookupGw: (
    id: string,
    type: 'entry' | 'exit',
    countryCode?: string,
  ) => Gateway | null;
};

export type GatewaysState = {
  mxEntry: GatewaysByCountry[];
  mxExit: GatewaysByCountry[];
  wg: GatewaysByCountry[];
  mxEntryLoading: boolean;
  mxExitLoading: boolean;
  wgLoading: boolean;
  mxEntryError: AppError | null;
  mxExitError: AppError | null;
  wgError: AppError | null;
};
