package net.nymtech.nymvpn.ui.model

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.ErrorStateReason
import nym_vpn_lib_types.EstablishConnectionState
import org.junit.Assert.assertEquals
import org.junit.Test

class MarkerStateTest {

	private fun from(state: Tunnel.State, killSwitch: Boolean = true, establish: EstablishConnectionState? = null) = Marker.from(state, establish, Tunnel.Mode.TWO_HOP_MIXNET, killSwitch)

	@Test
	fun from_staticStates_mapOneToOne() {
		assertEquals(Marker(MarkerState.NOT_PROTECTED, MarkerMode.DVPN), from(Tunnel.State.Down))
		assertEquals(Marker(MarkerState.CONNECTED, MarkerMode.DVPN), from(Tunnel.State.Up))
		assertEquals(Marker(MarkerState.STOPPING, MarkerMode.DVPN), from(Tunnel.State.Disconnecting))
	}

	@Test
	fun from_error_isError() {
		assertEquals(MarkerState.ERROR, from(Tunnel.State.Error(ErrorStateReason.TunnelProvider)).state)
	}

	@Test
	fun from_offlineMidSessionWithKillSwitch_isBlocked() {
		assertEquals(Marker(MarkerState.BLOCKED, MarkerMode.DVPN), from(Tunnel.State.Offline(reconnect = true)))
	}

	@Test
	fun from_offlineWithNothingToResume_isNotProtected() {
		assertEquals(Marker(MarkerState.NOT_PROTECTED, MarkerMode.DVPN), from(Tunnel.State.Offline(reconnect = false)))
	}

	@Test
	fun from_offlineWithKillSwitchOff_isNotProtected() {
		assertEquals(Marker(MarkerState.NOT_PROTECTED, MarkerMode.DVPN), from(Tunnel.State.Offline(reconnect = true), killSwitch = false))
	}

	@Test
	fun from_connectingPhases_drawSixSteps() {
		val expected = mapOf(
			EstablishConnectionState.RESOLVING_API_ADDRESSES to 1,
			EstablishConnectionState.AWAITING_ACCOUNT_READINESS to 2,
			EstablishConnectionState.AWAITING_CREDENTIALS_AVAILABILITY to 2,
			EstablishConnectionState.REFRESHING_GATEWAYS to 3,
			EstablishConnectionState.SELECTING_GATEWAYS to 4,
			EstablishConnectionState.REGISTERING_WITH_GATEWAYS to 5,
			EstablishConnectionState.CONNECTING_TUNNEL to 6,
		)
		expected.forEach { (phase, step) ->
			assertEquals(phase.name, Marker(MarkerState.CONNECTING, MarkerMode.DVPN, step), from(Tunnel.State.EstablishingConnection, establish = phase))
		}
		assertEquals(1, from(Tunnel.State.EstablishingConnection).step)
		assertEquals(1, from(Tunnel.State.InitializingClient).step)
	}

	@Test
	fun mode_tunnelMode_isTheFace() {
		assertEquals(MarkerMode.DVPN, Marker.mode(Tunnel.Mode.TWO_HOP_MIXNET))
		assertEquals(MarkerMode.MIXNET, Marker.mode(Tunnel.Mode.FIVE_HOP_MIXNET))
	}
}
