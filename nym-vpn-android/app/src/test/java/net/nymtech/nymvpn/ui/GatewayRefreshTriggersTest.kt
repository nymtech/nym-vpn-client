package net.nymtech.nymvpn.ui

import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.runBlocking
import net.nymtech.connectivity.NetworkStatus
import net.nymtech.vpn.backend.Tunnel
import org.junit.Assert.assertEquals
import org.junit.Test

class GatewayRefreshTriggersTest {

	@Test
	fun emitsModeOnceConnected() = runBlocking {
		val modes = flowOf(Tunnel.Mode.TWO_HOP_MIXNET)
		val network = flowOf(NetworkStatus.Connected)

		assertEquals(listOf(Tunnel.Mode.TWO_HOP_MIXNET), gatewayRefreshTriggers(modes, network).toList())
	}

	@Test
	fun doesNotEmitWhileDisconnected() = runBlocking {
		val modes = flowOf(Tunnel.Mode.FIVE_HOP_MIXNET)
		val network = flowOf(NetworkStatus.Disconnected)

		assertEquals(emptyList<Tunnel.Mode>(), gatewayRefreshTriggers(modes, network).toList())
	}

	@Test
	fun reEmitsSameModeWhenConnectivityReturns() = runBlocking {
		val modes = flowOf(Tunnel.Mode.FIVE_HOP_MIXNET)
		val network = flowOf(NetworkStatus.Connected, NetworkStatus.Disconnected, NetworkStatus.Connected)

		assertEquals(listOf(Tunnel.Mode.FIVE_HOP_MIXNET, Tunnel.Mode.FIVE_HOP_MIXNET), gatewayRefreshTriggers(modes, network).toList())
	}

	@Test
	fun emitsOnModeChangeWhileConnected() = runBlocking {
		val modes = flowOf(Tunnel.Mode.FIVE_HOP_MIXNET, Tunnel.Mode.FIVE_HOP_MIXNET, Tunnel.Mode.TWO_HOP_MIXNET)
		val network = flowOf(NetworkStatus.Connected)

		assertEquals(listOf(Tunnel.Mode.FIVE_HOP_MIXNET, Tunnel.Mode.TWO_HOP_MIXNET), gatewayRefreshTriggers(modes, network).toList())
	}
}
