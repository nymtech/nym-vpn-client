package net.nymtech.nymvpn.ui.model

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EstablishConnectionState

/**
 * Connection marker, state model (#7062, PR 1 of 5).
 * Maps the tunnel state onto the marker's six visual states. No UI reads this yet.
 */
enum class MarkerState { NOT_PROTECTED, CONNECTING, CONNECTED, ERROR, BLOCKED, STOPPING }

/** The face: two blocks for dVPN (2 hops), mix nodes for Mixnet. */
enum class MarkerMode { DVPN, MIXNET }

/** Why traffic is blocked, so the copy never promises a fix that can't work. */
enum class BlockedCause { NEEDS_RECONNECT, NO_INTERNET }

data class Marker(
	val state: MarkerState,
	val mode: MarkerMode,
	/** 1 to [CONNECTING_STEP_COUNT] while connecting, otherwise null. */
	val step: Int? = null,
	/** Set only when [state] is [MarkerState.BLOCKED]. */
	val blockedCause: BlockedCause? = null,
) {
	companion object {
		const val CONNECTING_STEP_COUNT = 6

		fun mode(tunnelMode: Tunnel.Mode): MarkerMode = when (tunnelMode) {
			Tunnel.Mode.FIVE_HOP_MIXNET -> MarkerMode.MIXNET
			Tunnel.Mode.TWO_HOP_MIXNET -> MarkerMode.DVPN
		}

		/**
		 * Core reports seven connecting phases; the marker draws six steps (three rings, each
		 * half then sealed). The two account checks are one step: "Checking your account".
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
		 * Kill switch on = Blocked, off = Not protected (Hux, 2026-11-13). On Android the kill
		 * switch is the system's always-on VPN, so callers pass `isVpnAlwaysOn(context)`.
		 */
		fun from(
			tunnelState: Tunnel.State,
			establish: EstablishConnectionState?,
			tunnelMode: Tunnel.Mode,
			killSwitch: Boolean,
		): Marker {
			val mode = mode(tunnelMode)
			return when (tunnelState) {
				Tunnel.State.Down -> Marker(MarkerState.NOT_PROTECTED, mode)
				Tunnel.State.Up -> Marker(MarkerState.CONNECTED, mode)
				Tunnel.State.Disconnecting -> Marker(MarkerState.STOPPING, mode)
				is Tunnel.State.Error -> Marker(MarkerState.ERROR, mode)
				Tunnel.State.InitializingClient -> Marker(MarkerState.CONNECTING, mode, step = 1)
				Tunnel.State.EstablishingConnection -> Marker(MarkerState.CONNECTING, mode, step = step(establish))
				is Tunnel.State.Offline -> when {
					!killSwitch -> Marker(MarkerState.NOT_PROTECTED, mode)
					tunnelState.reconnect -> Marker(MarkerState.BLOCKED, mode, blockedCause = BlockedCause.NEEDS_RECONNECT)
					else -> Marker(MarkerState.BLOCKED, mode, blockedCause = BlockedCause.NO_INTERNET)
				}
			}
		}
	}
}
