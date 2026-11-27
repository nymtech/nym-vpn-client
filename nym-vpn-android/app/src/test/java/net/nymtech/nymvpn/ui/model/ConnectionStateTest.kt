package net.nymtech.nymvpn.ui.model

import net.nymtech.connectivity.NetworkStatus
import net.nymtech.nymvpn.manager.backend.model.BackendUiEvent
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.VpnException
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.ErrorStateReason
import nym_vpn_lib_types.EstablishConnectionState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ConnectionStateTest {

	private fun step(tunnelState: Tunnel.State, establish: EstablishConnectionState? = null) = (ConnectionState.from(tunnelState, establish) as ConnectionState.Connecting).step

	private fun resolve(
		tunnelState: Tunnel.State,
		networkStatus: NetworkStatus = NetworkStatus.Connected,
		isRestarting: Boolean = false,
		event: BackendUiEvent? = null,
		accountState: AccountControllerState = AccountControllerState.Offline,
	) = ConnectionState.resolve(
		TunnelManagerState(tunnelState = tunnelState, isRestarting = isRestarting, backendUiEvent = event, accountState = accountState),
		networkStatus,
	)

	@Test
	fun from_establishPhases_mapToSixSteps() {
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
			assertEquals(phase.name, step, step(Tunnel.State.EstablishingConnection, phase))
		}
	}

	@Test
	fun from_beforeFirstPhase_isStepOne() {
		assertEquals(1, step(Tunnel.State.InitializingClient))
		assertEquals(1, step(Tunnel.State.EstablishingConnection))
	}

	@Test
	fun resolve_offline_waitsOnlyWhenReconnectArmed() {
		assertEquals(ConnectionState.WaitingForConnection, resolve(Tunnel.State.Offline(reconnect = true)))
		assertEquals(ConnectionState.Offline, resolve(Tunnel.State.Offline(reconnect = false)))
	}

	@Test
	fun resolve_networkDownWithLiveSession_isWaiting() {
		assertEquals(ConnectionState.WaitingForConnection, resolve(Tunnel.State.Up, NetworkStatus.Disconnected))
		assertEquals(ConnectionState.WaitingForConnection, resolve(Tunnel.State.Up, NetworkStatus.Disconnected, isRestarting = true))
	}

	@Test
	fun resolve_networkDownWithoutSession_isOffline() {
		assertEquals(ConnectionState.Offline, resolve(Tunnel.State.Down, NetworkStatus.Disconnected))
		assertEquals(ConnectionState.Offline, resolve(Tunnel.State.EstablishingConnection, NetworkStatus.Disconnected, isRestarting = true))
	}

	@Test
	fun resolve_networkDownWhileConnecting_keepsStep() {
		val state = resolve(Tunnel.State.EstablishingConnection, NetworkStatus.Disconnected)

		assertTrue(state is ConnectionState.Connecting)
	}

	@Test
	fun resolve_restartWithTunnelDown_isDisconnecting() {
		assertEquals(ConnectionState.Disconnecting, resolve(Tunnel.State.Down, isRestarting = true))
	}

	@Test
	fun resolve_failureEvent_isError() {
		val reason = ErrorStateReason.TunnelProvider

		assertEquals(ConnectionState.Error(reason), resolve(Tunnel.State.Down, event = BackendUiEvent.Failure(reason)))
	}

	@Test
	fun resolve_relaxedCriteriaFailure_keepsBaseState() {
		assertEquals(ConnectionState.Disconnected, resolve(Tunnel.State.Down, event = BackendUiEvent.Failure(ErrorStateReason.NeedsRelaxedIndependenceCriteria)))
	}

	@Test
	fun resolve_subscriptionFailureWithReadyAccount_keepsBaseState() {
		val event = BackendUiEvent.Failure(ErrorStateReason.InactiveSubscription)

		assertEquals(ConnectionState.Disconnected, resolve(Tunnel.State.Down, event = event, accountState = AccountControllerState.ReadyToConnect))
		assertEquals(ConnectionState.Error(ErrorStateReason.InactiveSubscription), resolve(Tunnel.State.Down, event = event))
	}

	@Test
	fun resolve_startFailure_isStartFailure() {
		val exception = VpnException.InternalException("start")

		assertEquals(ConnectionState.StartFailure(exception), resolve(Tunnel.State.Down, event = BackendUiEvent.StartFailure(exception)))
	}

	@Test
	fun isFailure_coversErrorAndStartFailure() {
		assertTrue(ConnectionState.Error(ErrorStateReason.TunnelProvider).isFailure)
		assertTrue(ConnectionState.StartFailure(VpnException.InternalException("start")).isFailure)
		assertFalse(ConnectionState.Disconnected.isFailure)
	}
}
