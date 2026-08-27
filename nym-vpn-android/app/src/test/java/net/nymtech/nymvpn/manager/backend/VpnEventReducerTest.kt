package net.nymtech.nymvpn.manager.backend

import net.nymtech.nymvpn.manager.backend.model.ConnectionInfo
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EstablishConnectionState
import nym_vpn_lib_types.GatewayLightInfo
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class VpnEventReducerTest {

	private fun gateway(id: String) = GatewayLightInfo(id, "FR")

	private fun info(connectedAt: Long?) = ConnectionInfo(gateway("entry"), gateway("exit"), connectedAt, null)

	@Test
	fun establishConnection_preservesSessionStart() {
		val current = TunnelManagerState(tunnelState = Tunnel.State.EstablishingConnection, connectionData = info(1000L))

		val next = VpnEventReducer.reduceEstablishConnection(current, EstablishConnectionState.RESOLVING_API_ADDRESSES, info(null))

		assertEquals(1000L, next.connectionData?.connectedAt)
	}

	@Test
	fun establishConnection_withoutEventData_keepsPreviousConnectionData() {
		val current = TunnelManagerState(tunnelState = Tunnel.State.EstablishingConnection, connectionData = info(1000L))

		val next = VpnEventReducer.reduceEstablishConnection(current, EstablishConnectionState.RESOLVING_API_ADDRESSES, null)

		assertEquals(1000L, next.connectionData?.connectedAt)
	}

	@Test
	fun connected_preservesSessionStartAcrossOfflineReconnect() {
		// Offline auto-reconnect does not set isRestarting; the session start must survive anyway
		val current = TunnelManagerState(tunnelState = Tunnel.State.Up, connectionData = info(1000L), isRestarting = false)

		val next = VpnEventReducer.reduceConnected(current, info(2000L))

		assertEquals(1000L, next.connectionData?.connectedAt)
	}

	@Test
	fun connected_withoutEventData_keepsPreviousConnectionData() {
		val current = TunnelManagerState(tunnelState = Tunnel.State.Up, connectionData = info(1000L))

		val next = VpnEventReducer.reduceConnected(current, null)

		assertEquals(1000L, next.connectionData?.connectedAt)
	}

	@Test
	fun connected_freshSession_usesNewConnectedAt() {
		val current = TunnelManagerState(tunnelState = Tunnel.State.Up, connectionData = null)

		val next = VpnEventReducer.reduceConnected(current, info(2000L))

		assertEquals(2000L, next.connectionData?.connectedAt)
	}

	@Test
	fun stateChangedDown_clearsConnectionData() {
		val current = TunnelManagerState(tunnelState = Tunnel.State.Up, connectionData = info(1000L))

		val next = VpnEventReducer.reduceStateChanged(current, Tunnel.State.Down)

		assertNull(next.connectionData)
	}
}
