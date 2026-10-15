package net.nymtech.nymvpn.data.file

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.coroutines.yield
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.nymvpn.testutil.gateway
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Assert.assertNull
import org.junit.Assume.assumeTrue
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

	@Test
	fun concurrentWritesToSameListPersistTheFinalState() = runBlocking {
		val repo = FileGatewayRepository(tmp.root, Dispatchers.IO)

		withContext(Dispatchers.Default) {
			(1..40).map { n -> async { repo.setEntryGateways((1..n).map { gateway("e$it") }) } }.awaitAll()
		}

		val inMemory = repo.gatewayFlow.first().entryGateways
		val onDisk = repository().gatewayFlow.first().entryGateways
		assertEquals(inMemory, onDisk)
		assertEquals(emptyList<String>(), tmp.root.list()!!.filter { it.endsWith(".tmp") })
	}

	@Test
	fun failedWriteIsRetriedOnNextIdenticalRefresh() = runBlocking {
		val repo = repository()
		val entry = listOf(gateway("e1"))
		assumeTrue(tmp.root.setWritable(false))
		try {
			repo.setEntryGateways(entry)
		} finally {
			tmp.root.setWritable(true)
		}
		assertEquals(entry, repo.gatewayFlow.first().entryGateways)
		assertEquals(Gateways(), repository().gatewayFlow.first())

		repo.setEntryGateways(entry.map { it.copy() })

		assertEquals(entry, repository().gatewayFlow.first().entryGateways)
	}

	@Test
	fun importLegacySeedsEmptyListsAndPersistsThem() = runBlocking {
		val entry = listOf(gateway("e1"))
		val wg = listOf(gateway("w1"))

		assertTrue(repository().importLegacy(entry = entry, exit = emptyList(), wg = wg))

		val reloaded = repository().gatewayFlow.first()
		assertEquals(entry, reloaded.entryGateways)
		assertEquals(wg, reloaded.wgGateways)
		assertEquals(emptyList<Any>(), reloaded.exitGateways)
	}

	@Test
	fun importLegacyDoesNotOverwriteListsAlreadyPresent() = runBlocking {
		val existing = listOf(gateway("x1"))
		repository().setEntryGateways(existing)

		assertTrue(repository().importLegacy(entry = listOf(gateway("e1")), exit = emptyList(), wg = emptyList()))

		assertEquals(existing, repository().gatewayFlow.first().entryGateways)
	}

	@Test
	fun importLegacyReturnsFalseWhenNothingCouldBePersisted() = runBlocking {
		val repo = repository()
		assumeTrue(tmp.root.setWritable(false))
		try {
			assertFalse(repo.importLegacy(entry = listOf(gateway("e1")), exit = emptyList(), wg = emptyList()))
		} finally {
			tmp.root.setWritable(true)
		}
	}
}
