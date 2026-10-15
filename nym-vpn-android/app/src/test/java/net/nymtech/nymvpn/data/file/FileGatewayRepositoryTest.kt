package net.nymtech.nymvpn.data.file

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.coroutines.yield
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.testutil.gateway
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

class FileGatewayRepositoryTest {

	@get:Rule
	val tmp = TemporaryFolder()

	private fun repository() = FileGatewayRepository(tmp.root, Dispatchers.Unconfined)

	@Test
	fun emptyWhenNothingStored() = runBlocking {
		assertEquals(Gateways(), repository().gatewayFlow.first())
	}

	@Test
	fun persistedListsSurviveNewInstance() = runBlocking {
		val entry = listOf(gateway("e1"), gateway("e2", country = "de"))
		val wg = listOf(gateway("w1"))
		repository().apply {
			setEntryGateways(entry)
			setWgGateways(wg)
		}

		val reloaded = repository().gatewayFlow.first()

		assertEquals(entry, reloaded.entryGateways)
		assertEquals(wg, reloaded.wgGateways)
		assertEquals(emptyList<Any>(), reloaded.exitGateways)
	}

	@Test
	fun identicalWriteDoesNotEmitAgain() = runBlocking {
		val repo = repository()
		val entry = listOf(gateway("e1"))
		repo.setEntryGateways(entry)

		val emissions = mutableListOf<Gateways>()
		val job = launch { repo.gatewayFlow.collect { emissions.add(it) } }
		yield()
		repo.setEntryGateways(entry.map { it.copy() })
		yield()
		repo.setExitGateways(listOf(gateway("x1")))
		yield()
		job.cancel()

		assertEquals(2, emissions.size)
		assertEquals(entry, emissions[0].entryGateways)
		assertEquals("x1", emissions[1].exitGateways.single().identity)
	}

	@Test
	fun corruptFileFallsBackToEmptyList() = runBlocking {
		repository().setExitGateways(listOf(gateway("x1")))
		tmp.root.listFiles()!!.single { it.name.startsWith("exit") }.writeText("{not json")

		assertEquals(Gateways(), repository().gatewayFlow.first())
	}

	@Test
	fun flowDoesNotEmitBeforeDiskLoadCompletes() = runBlocking {
		repository().setEntryGateways(listOf(gateway("e1")))

		val first = withTimeoutOrNull(2_000) { repository().gatewayFlow.take(1).toList().single() }

		assertEquals("e1", first?.entryGateways?.single()?.identity)
		assertNull(first?.entryGateways?.firstOrNull { it.identity != "e1" })
	}
}
