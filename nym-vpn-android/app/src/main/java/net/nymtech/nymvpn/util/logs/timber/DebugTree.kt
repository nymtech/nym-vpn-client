package net.nymtech.nymvpn.util.logs.timber

import timber.log.Timber

class DebugTree(private val minPriority: Int) : Timber.DebugTree() {

	override fun isLoggable(tag: String?, priority: Int): Boolean = priority >= minPriority
}
