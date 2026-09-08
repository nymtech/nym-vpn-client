package net.nymtech.nymvpn.ui.screens.main

import net.nymtech.nymvpn.ui.screens.main.ConnectionTimerPolicy.Command
import net.nymtech.vpn.backend.Tunnel
import org.junit.Assert.assertEquals
import org.junit.Test

class ConnectionTimerPolicyTest {

	@Test
	fun up_withSessionStart_startsTimer() {
		assertEquals(Command.Start(1000L), ConnectionTimerPolicy.evaluate(Tunnel.State.Up, 1000L))
	}

	@Test
	fun up_withoutSessionStart_leavesTimerUntouched() {
		assertEquals(Command.None, ConnectionTimerPolicy.evaluate(Tunnel.State.Up, null))
	}

	@Test
	fun offline_stopsTimer_evenWithSessionActive() {
		assertEquals(Command.Stop, ConnectionTimerPolicy.evaluate(Tunnel.State.Offline(reconnect = true), 1000L))
		assertEquals(Command.Stop, ConnectionTimerPolicy.evaluate(Tunnel.State.Offline(reconnect = false), 1000L))
	}

	@Test
	fun offline_withoutSession_stopsTimer() {
		assertEquals(Command.Stop, ConnectionTimerPolicy.evaluate(Tunnel.State.Offline(reconnect = false), null))
	}

	@Test
	fun establishing_stopsTimer_untilTunnelIsUp() {
		assertEquals(Command.Stop, ConnectionTimerPolicy.evaluate(Tunnel.State.EstablishingConnection, 1000L))
		assertEquals(Command.Stop, ConnectionTimerPolicy.evaluate(Tunnel.State.InitializingClient, 1000L))
	}

	@Test
	fun down_disconnecting_error_stopTimer() {
		assertEquals(Command.Stop, ConnectionTimerPolicy.evaluate(Tunnel.State.Down, 1000L))
		assertEquals(Command.Stop, ConnectionTimerPolicy.evaluate(Tunnel.State.Disconnecting, 1000L))
	}
}
