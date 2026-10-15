package net.nymtech.nymvpn.service.gateway

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.runBlocking
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.testutil.gateway
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.GatewayType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class TtlGatewayCacheServiceTest {

	private class FakeRepository : GatewayRepository {
		val state = MutableStateFlow(Gateways())
		override val gatewayFlow: Flow<Gateways> = state
		override suspend fun setEntryGateways(gateways: List<NymGateway>) = state.update { it.copy(entryGateways = gateways) }
		override suspend fun setExitGateways(gateways: List<NymGateway>) = state.update { it.copy(exitGateways = gateways) }
		override suspend fun setWgGateways(gateways: List<NymGateway>) = state.update { it.copy(wgGateways = gateways) }
		private fun MutableStateFlow<Gateways>.update(block: (Gateways) -> Gateways) {
			value = block(value)
		}
	}

	private var now = 0L
	private val fetches = mutableListOf<GatewayType>()
	private var failNext = false
	private val repository = FakeRepository()
	private val service = TtlGatewayCacheService(
		repository = repository,
		fetch = { type ->
			fetches.add(type)
			if (failNext) {
				failNext = false
				error("boom")
			}
			listOf(gateway("g-${type.name}"))
		},
		ttlMs = 1_000,
		nowMs = { now },
	)

	@Test
	fun firstRefreshFetchesAndStores() = runBlocking {
		val result = service.refresh(GatewayType.WG)

		assertTrue(result.isSuccess)
		assertEquals(listOf(GatewayType.WG), fetches)
		assertEquals("g-WG", repository.state.value.wgGateways.single().identity)
	}

	@Test
	fun refreshWithinTtlIsSkipped() = runBlocking {
		service.refresh(GatewayType.MIXNET_ENTRY)
		now = 999
		service.refresh(GatewayType.MIXNET_ENTRY)

		assertEquals(1, fetches.size)
	}

	@Test
	fun refreshAfterTtlFetchesAgain() = runBlocking {
		service.refresh(GatewayType.MIXNET_ENTRY)
		now = 1_000
		service.refresh(GatewayType.MIXNET_ENTRY)

		assertEquals(2, fetches.size)
	}

	@Test
	fun forcedRefreshIgnoresTtl() = runBlocking {
		service.refresh(GatewayType.MIXNET_EXIT)
		service.refresh(GatewayType.MIXNET_EXIT, force = true)

		assertEquals(2, fetches.size)
	}

	@Test
	fun failedFetchDoesNotStartTtl() = runBlocking {
		failNext = true
		val failed = service.refresh(GatewayType.WG)
		val retried = service.refresh(GatewayType.WG)

		assertTrue(failed.isFailure)
		assertTrue(retried.isSuccess)
		assertEquals(2, fetches.size)
	}

	@Test
	fun ttlIsTrackedPerType() = runBlocking {
		service.refresh(GatewayType.MIXNET_ENTRY)
		service.refresh(GatewayType.MIXNET_EXIT)

		assertEquals(listOf(GatewayType.MIXNET_ENTRY, GatewayType.MIXNET_EXIT), fetches)
	}

	@Test
	fun refreshForModeFetchesOnlyListsUsedByThatMode() = runBlocking {
		service.refreshForMode(Tunnel.Mode.TWO_HOP_MIXNET)
		assertEquals(listOf(GatewayType.WG), fetches)

		fetches.clear()
		service.refreshForMode(Tunnel.Mode.FIVE_HOP_MIXNET)
		assertEquals(setOf(GatewayType.MIXNET_ENTRY, GatewayType.MIXNET_EXIT), fetches.toSet())
	}

	@Test
	fun refreshForModeReportsFailureOfAnyList() = runBlocking {
		failNext = true
		val failed = service.refreshForMode(Tunnel.Mode.FIVE_HOP_MIXNET)
		val ok = service.refreshForMode(Tunnel.Mode.FIVE_HOP_MIXNET)

		assertTrue(failed.isFailure)
		assertTrue(ok.isSuccess)
	}
}
