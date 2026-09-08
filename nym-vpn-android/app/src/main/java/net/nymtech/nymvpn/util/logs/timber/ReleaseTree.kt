package net.nymtech.nymvpn.util.logs.timber

import timber.log.Timber

class ReleaseTree(private val minPriority: Int) : Timber.DebugTree() {

	override fun isLoggable(tag: String?, priority: Int): Boolean = priority >= minPriority

	override fun d(t: Throwable?) {
		return
	}

	override fun d(t: Throwable?, message: String?, vararg args: Any?) {
		return
	}

	override fun d(message: String?, vararg args: Any?) {
		return
	}
}
