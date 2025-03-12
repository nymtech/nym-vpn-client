import { AppError, GatewayType, GatewaysByCountry } from '../../types';

export type FetchGatewaysFn = (
  nodeType: GatewayType,
) => Promise<void> | undefined;

export type GatewaysState = {
  mxEntry: GatewaysByCountry[];
  mxExit: GatewaysByCountry[];
  wg: GatewaysByCountry[];
  mxEntryLoading: boolean;
  mxExitLoading: boolean;
  wgLoading: boolean;
  mxEntryError?: AppError | null;
  mxExitError?: AppError | null;
  wgError?: AppError | null;
  fetch: FetchGatewaysFn;
};
