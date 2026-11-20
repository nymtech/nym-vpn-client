package net.nymtech.nymvpn.ui.model

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EstablishConnectionState

/**
 * Connection marker state model (#6909): maps the tunnel state onto the marker's states.
 * Covers five of the six design states; connected with data comes with the data line.
 */
enum class MarkerState { NOT_PROTECTED, CONNECTING, CONNECTED, ERROR, BLOCKED, STOPPING }

/** The face: two blocks for dVPN (2 hops), the mix field for Mixnet. */
enum class MarkerMode { DVPN, MIXNET }

data class Marker(
	val state: MarkerState,
	val mode: MarkerMode,
	/** 1 to [CONNECTING_STEP_COUNT] while connecting, otherwise null. */
	val step: Int? = null,
) {
	companion object {
		const val CONNECTING_STEP_COUNT = 6

		fun mode(tunnelMode: Tunnel.Mode): MarkerMode = when (tunnelMode) {
			Tunnel.Mode.FIVE_HOP_MIXNET -> MarkerMode.MIXNET
			Tunnel.Mode.TWO_HOP_MIXNET -> MarkerMode.DVPN
		}

		/**
		 * Core reports seven connecting phases; the marker draws six steps (three rings, each
		 * half then sealed). The two account checks share step 2.
		 */
		fun step(establish: EstablishConnectionState?): Int = when (establish) {
			null, EstablishConnectionState.RESOLVING_API_ADDRESSES -> 1
			EstablishConnectionState.AWAITING_ACCOUNT_READINESS,
			EstablishConnectionState.AWAITING_CREDENTIALS_AVAILABILITY,
			-> 2
			EstablishConnectionState.REFRESHING_GATEWAYS -> 3
			EstablishConnectionState.SELECTING_GATEWAYS -> 4
			EstablishConnectionState.REGISTERING_WITH_GATEWAYS -> 5
			EstablishConnectionState.CONNECTING_TUNNEL -> 6
		}

		/**
		 * Blocked only while traffic is really held: offline mid-session (reconnect armed) with the
		 * kill switch on. Offline with nothing to resume leaves networking open, so it is not
		 * protected. [killSwitch] is the system's always-on VPN setting.
		 */
		fun from(tunnelState: Tunnel.State, establish: EstablishConnectionState?, tunnelMode: Tunnel.Mode, killSwitch: Boolean): Marker {
			val mode = mode(tunnelMode)
			return when (tunnelState) {
				Tunnel.State.Down -> Marker(MarkerState.NOT_PROTECTED, mode)
				Tunnel.State.Up -> Marker(MarkerState.CONNECTED, mode)
				Tunnel.State.Disconnecting -> Marker(MarkerState.STOPPING, mode)
				is Tunnel.State.Error -> Marker(MarkerState.ERROR, mode)
				Tunnel.State.InitializingClient -> Marker(MarkerState.CONNECTING, mode, step = 1)
				Tunnel.State.EstablishingConnection -> Marker(MarkerState.CONNECTING, mode, step = step(establish))
				is Tunnel.State.Offline -> if (tunnelState.reconnect && killSwitch) Marker(MarkerState.BLOCKED, mode) else Marker(MarkerState.NOT_PROTECTED, mode)
			}
		}
	}
}
