import type { ConnectingProgress, TunnelState, VpnMode } from '../../../types';

// Connection marker, state model (#7064, PR 1 of 5).
// Maps the daemon's tunnel state onto the marker's six visual states.
// No UI reads this yet; PR 2 draws from it.

export type MarkerState =
  | 'not-protected'
  | 'connecting'
  | 'connected'
  | 'error'
  | 'blocked'
  // not one of the six Figma variants: a short fade out of the current state
  | 'stopping';

// the face: two blocks for dVPN (2 hops), mix nodes for Mixnet
export type MarkerMode = 'dvpn' | 'mixnet';

// why traffic is blocked, so the copy never promises a fix that can't work
export type BlockedCause = 'needs-reconnect' | 'no-internet';

export const CONNECTING_STEP_COUNT = 6;

export type Marker = {
  state: MarkerState;
  mode: MarkerMode;
  // 1 to CONNECTING_STEP_COUNT while connecting, otherwise null
  step: number | null;
  // set only when state is 'blocked'
  blockedCause: BlockedCause | null;
};

// Core reports seven connecting phases; the marker draws six steps
// (three rings, each half then sealed). The two account checks are one
// step for the person: "Checking your account".
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
  blockedCause: BlockedCause | null = null,
): Marker {
  return { state, mode, step, blockedCause };
}

// Desktop's kill switch is always on (no user toggle), so offline means
// traffic is held: Blocked. The rule is kill switch on = Blocked,
// off = Not protected; mobile passes its toggle, desktop keeps the default.
export function markerFromTunnel(
  state: TunnelState,
  vpnMode: VpnMode,
  progress?: ConnectingProgress | null,
  killSwitch = true,
): Marker {
  const mode = markerMode(vpnMode);
  switch (state) {
    case 'disconnected':
    case 'unknown':
      return marker('not-protected', mode);
    case 'connecting':
      return marker(
        'connecting',
        mode,
        progress ? CONNECTING_STEP[progress] : 1,
      );
    case 'connected':
      return marker('connected', mode);
    case 'disconnecting':
      return marker('stopping', mode);
    case 'error':
      return marker('error', mode);
    case 'offline-auto-reconnect':
      return killSwitch
        ? marker('blocked', mode, null, 'needs-reconnect')
        : marker('not-protected', mode);
    case 'offline':
      return killSwitch
        ? marker('blocked', mode, null, 'no-internet')
        : marker('not-protected', mode);
  }
}
