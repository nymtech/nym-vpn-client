package net.nymtech.nymvpn.util.extensions

import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleRegistry

/**
 * TileService delivers its callbacks through a handler, so a queued onStartListening/onStopListening
 * can arrive after onDestroy. Lifecycle 2.9+ throws IllegalStateException when moving a DESTROYED
 * registry, so ignore events once destroyed.
 */
fun LifecycleRegistry.handleLifecycleEventSafely(event: Lifecycle.Event) {
	if (currentState == Lifecycle.State.DESTROYED) return
	handleLifecycleEvent(event)
}
