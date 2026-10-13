package net.nymtech.nymvpn.manager.billing

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import kotlinx.coroutines.yield
import net.nymtech.nymvpn.manager.backend.BackendManager
import org.junit.Assert.assertEquals
import org.junit.Test
import java.lang.reflect.Proxy
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds

class TrialEligibilityRepositoryTest {

	@Test
	fun noStoredAccount_isEligibleWithoutAskingBackend() = runBlocking {
		val backend = FakeBackend(accountStored = false) { error("backend must not be called") }

		assertEquals(TrialEligibility.Resolved(eligible = true), resolve(backend))
		assertEquals(0, backend.trialEligibleCalls)
	}

	@Test
	fun storedAccount_backendSaysEligible() = runBlocking {
		val backend = FakeBackend(accountStored = true) { true }

		assertEquals(TrialEligibility.Resolved(eligible = true), resolve(backend))
		assertEquals(1, backend.trialEligibleCalls)
	}

	@Test
	fun storedAccount_backendSaysNotEligible() = runBlocking {
		assertEquals(TrialEligibility.Resolved(eligible = false), resolve(FakeBackend(accountStored = true) { false }))
	}

	@Test
	fun backendThrows_isNotEligible() = runBlocking {
		assertEquals(TrialEligibility.Resolved(eligible = false), resolve(FakeBackend(accountStored = true) { error("boom") }))
	}

	@Test
	fun storedAccountCheckThrows_isNotEligible() = runBlocking {
		val backend = FakeBackend(accountStored = true, accountStoredError = IllegalStateException("service not bound")) { true }

		assertEquals(TrialEligibility.Resolved(eligible = false), resolve(backend))
	}

	@Test
	fun backendTimesOut_isNotEligible() = runBlocking {
		val backend = FakeBackend(accountStored = true) { awaitCancellation() }

		assertEquals(TrialEligibility.Resolved(eligible = false), resolve(backend, timeout = 50.milliseconds))
	}

	@Test
	fun staleAnswerDoesNotOverwriteNewerOne() = runBlocking {
		val older = CompletableDeferred<Boolean>()
		val newer = CompletableDeferred<Boolean>()
		val answers = ArrayDeque(listOf(older, newer))
		// NonCancellable: the stale call still delivers its answer.
		val backend = FakeBackend(accountStored = true) { withContext(NonCancellable) { answers.removeFirst().await() } }
		val repository = TrialEligibilityRepository(backend, 5.seconds)

		repository.refresh(this)
		yield()
		repository.refresh(this)
		yield()
		assertEquals(TrialEligibility.Loading, repository.state.value)

		newer.complete(false)
		assertEquals(TrialEligibility.Resolved(eligible = false), repository.state.first { it is TrialEligibility.Resolved })

		older.complete(true)
		repeat(10) { yield() }
		assertEquals(TrialEligibility.Resolved(eligible = false), repository.state.value)
	}

	@Test
	fun refreshGoesBackToLoadingAndReadsAgain() = runBlocking {
		var answer = true
		val backend = FakeBackend(accountStored = true) { answer }
		val repository = TrialEligibilityRepository(backend, 5.seconds)

		assertEquals(TrialEligibility.Resolved(eligible = true), repository.resolveIn(this))
		answer = false
		assertEquals(TrialEligibility.Resolved(eligible = false), repository.resolveIn(this))
		assertEquals(2, backend.trialEligibleCalls)
	}

	private suspend fun CoroutineScope.resolve(backend: BackendManager, timeout: Duration = 5.seconds): TrialEligibility = TrialEligibilityRepository(backend, timeout).resolveIn(this)

	private suspend fun TrialEligibilityRepository.resolveIn(scope: CoroutineScope): TrialEligibility {
		refresh(scope)
		assertEquals(TrialEligibility.Loading, state.value)
		return state.first { it is TrialEligibility.Resolved }
	}
}

private val unsupportedBackend = Proxy.newProxyInstance(
	BackendManager::class.java.classLoader,
	arrayOf(BackendManager::class.java),
) { _, method, _ -> throw UnsupportedOperationException(method.name) } as BackendManager

private class FakeBackend(private val accountStored: Boolean, private val accountStoredError: Exception? = null, private val trialEligible: suspend () -> Boolean) :
	BackendManager by unsupportedBackend {
	var trialEligibleCalls = 0
		private set

	override suspend fun isMnemonicStored(): Boolean {
		accountStoredError?.let { throw it }
		return accountStored
	}

	override suspend fun isTrialEligible(): Boolean {
		trialEligibleCalls++
		return trialEligible()
	}
}
