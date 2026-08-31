package net.nymtech.nymvpn.util.extensions

import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleRegistry

/** Ignores the event instead of throwing if the registry is already DESTROYED. */
fun LifecycleRegistry.handleLifecycleEventSafely(event: Lifecycle.Event) {
	if (currentState == Lifecycle.State.DESTROYED) return
	handleLifecycleEvent(event)
}
