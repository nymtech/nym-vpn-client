package net.nymtech.nymvpn.util.extensions

import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.LifecycleRegistry
import org.junit.Assert.assertEquals
import org.junit.Test

class LifecycleExtensionsTest {

	private fun registry(): LifecycleRegistry {
		lateinit var registry: LifecycleRegistry
		val owner = object : LifecycleOwner {
			override val lifecycle: Lifecycle
				get() = registry
		}
		registry = LifecycleRegistry.createUnsafe(owner)
		return registry
	}

	@Test
	fun eventsBeforeDestroy_moveStateNormally() {
		val registry = registry()

		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_CREATE)
		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_START)

		assertEquals(Lifecycle.State.STARTED, registry.currentState)
	}

	@Test
	fun stopAfterStart_movesBackToCreated() {
		val registry = registry()

		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_CREATE)
		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_START)
		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_STOP)

		assertEquals(Lifecycle.State.CREATED, registry.currentState)
	}

	@Test
	fun startAfterDestroy_isIgnored() {
		val registry = registry()
		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_CREATE)
		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_START)
		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_STOP)
		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_DESTROY)

		// simulates a queued onStartListening arriving after onDestroy
		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_START)

		assertEquals(Lifecycle.State.DESTROYED, registry.currentState)
	}

	@Test
	fun stopAfterDestroy_isIgnored() {
		val registry = registry()
		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_CREATE)
		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_START)
		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_DESTROY)

		registry.handleLifecycleEventSafely(Lifecycle.Event.ON_STOP)

		assertEquals(Lifecycle.State.DESTROYED, registry.currentState)
	}
}
