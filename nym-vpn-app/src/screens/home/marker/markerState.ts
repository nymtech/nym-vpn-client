import type { ConnectingProgress, TunnelState, VpnMode } from '../../../types';

// Connection marker state model (#6909): maps the daemon's tunnel state onto
// the marker's states. Covers five of the six design states; connected with
// data comes with the data line.

export type MarkerState =
  | 'not-protected'
  | 'connecting'
  | 'connected'
  | 'error'
  | 'blocked'
  // not one of the design states: a short fade out of the current state
  | 'stopping';

// the face: two blocks for dVPN (2 hops), the mix field for Mixnet
export type MarkerMode = 'dvpn' | 'mixnet';

export const CONNECTING_STEP_COUNT = 6;

export type Marker = {
  state: MarkerState;
  mode: MarkerMode;
  // 1 to CONNECTING_STEP_COUNT while connecting, otherwise null
  step: number | null;
};

// Core reports seven connecting phases; the marker draws six steps (three
// rings, each half then sealed). The two account checks share step 2.
const CONNECTING_STEP: Record<ConnectingProgress, number> = {
  'resolving-api-addresses': 1,
  'awaiting-account-readiness': 2,
  'awaiting-credentials-availability': 2,
  'refreshing-gateways': 3,
  'selecting-gateways': 4,
  'registering-with-gateways': 5,
  'connecting-tunnel': 6,
};

export function markerMode(vpnMode: VpnMode): MarkerMode {
  return vpnMode === 'mixnet' ? 'mixnet' : 'dvpn';
}

function marker(
  state: MarkerState,
  mode: MarkerMode,
  step: number | null = null,
): Marker {
  return { state, mode, step };
}

// Blocked only while traffic is really held: offline mid-session (reconnect
// armed). The desktop kill switch is always on. Offline with nothing to
// resume leaves networking open, so it is not protected.
export function markerFromTunnel(
  state: TunnelState,
  vpnMode: VpnMode,
  progress?: ConnectingProgress | null,
): Marker {
  const mode = markerMode(vpnMode);
  switch (state) {
    case 'disconnected':
    case 'unknown':
    case 'offline':
      return marker('not-protected', mode);
    case 'connecting':
      // an unknown phase (daemon newer than the app) starts at step 1
      return marker(
        'connecting',
        mode,
        progress ? (CONNECTING_STEP[progress] ?? 1) : 1,
      );
    case 'connected':
      return marker('connected', mode);
    case 'disconnecting':
      return marker('stopping', mode);
    case 'error':
      return marker('error', mode);
    case 'offline-auto-reconnect':
      return marker('blocked', mode);
  }
}
