import { AppError, GatewaysByCountry } from '../../types';

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
