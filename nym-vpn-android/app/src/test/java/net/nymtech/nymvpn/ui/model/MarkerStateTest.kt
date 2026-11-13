package net.nymtech.nymvpn.ui.model

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EstablishConnectionState
import org.junit.Assert.assertEquals
import org.junit.Test

class MarkerStateTest {

	private fun from(state: Tunnel.State, killSwitch: Boolean = true, establish: EstablishConnectionState? = null) =
		Marker.from(state, establish, Tunnel.Mode.TWO_HOP_MIXNET, killSwitch)

	@Test
	fun staticStates() {
		assertEquals(MarkerState.NOT_PROTECTED, from(Tunnel.State.Down).state)
		assertEquals(MarkerState.CONNECTED, from(Tunnel.State.Up).state)
		assertEquals(MarkerState.STOPPING, from(Tunnel.State.Disconnecting).state)
	}

	@Test
	fun offlineFollowsKillSwitch() {
		assertEquals(
			Marker(MarkerState.BLOCKED, MarkerMode.DVPN, blockedCause = BlockedCause.NO_INTERNET),
			from(Tunnel.State.Offline(reconnect = false)),
		)
		assertEquals(
			Marker(MarkerState.BLOCKED, MarkerMode.DVPN, blockedCause = BlockedCause.NEEDS_RECONNECT),
			from(Tunnel.State.Offline(reconnect = true)),
		)
		assertEquals(MarkerState.NOT_PROTECTED, from(Tunnel.State.Offline(reconnect = false), killSwitch = false).state)
		assertEquals(MarkerState.NOT_PROTECTED, from(Tunnel.State.Offline(reconnect = true), killSwitch = false).state)
	}

	@Test
	fun sevenPhasesDrawSixSteps() {
		val expected = mapOf(
			EstablishConnectionState.RESOLVING_API_ADDRESSES to 1,
			EstablishConnectionState.AWAITING_ACCOUNT_READINESS to 2,
			EstablishConnectionState.AWAITING_CREDENTIALS_AVAILABILITY to 2,
			EstablishConnectionState.REFRESHING_GATEWAYS to 3,
			EstablishConnectionState.SELECTING_GATEWAYS to 4,
			EstablishConnectionState.REGISTERING_WITH_GATEWAYS to 5,
			EstablishConnectionState.CONNECTING_TUNNEL to 6,
		)
		expected.forEach { (phase, step) -> assertEquals(phase.name, step, Marker.step(phase)) }
		assertEquals(1, Marker.step(null))
		assertEquals(1, from(Tunnel.State.InitializingClient).step)
	}

	@Test
	fun modeIsTheFace() {
		assertEquals(MarkerMode.DVPN, Marker.mode(Tunnel.Mode.TWO_HOP_MIXNET))
		assertEquals(MarkerMode.MIXNET, Marker.mode(Tunnel.Mode.FIVE_HOP_MIXNET))
	}
}
