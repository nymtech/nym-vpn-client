import {
  AppError,
  Gateway,
  GatewayType,
  GatewaysByCountry,
  RecentGateways,
  VpnMode,
} from '../../../types';

export type GatewaysSlice = GatewaysState & {
  fetchGateways: (nodeType: GatewayType) => Promise<void>;
  fetchRecents: (vpnMode: VpnMode) => Promise<void>;
  lookupGw: (
    id: string,
    type: 'entry' | 'exit',
    countryCode?: string,
  ) => Gateway | null;
};

/** The country-grouped gateway lists, one per gateway type. */
export type GatewayListsState = {
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

export type RecentsState = {
  /**
   * Most recently connected gateways per mode, as reported by the daemon,
   * ordered most-recent-first.
   *
   * Not cached: the daemon only appends here on a successful connection —
   * exactly when the user is most likely to open the list — so a TTL hides the
   * gateway they just used.
   */
  recents: Record<VpnMode, RecentGateways>;
  recentsLoading: Record<VpnMode, boolean>;
  recentsError: Record<VpnMode, AppError | null>;
};

export type GatewaysState = GatewayListsState & RecentsState;
