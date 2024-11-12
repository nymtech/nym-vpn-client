package net.nymtech.nymvpn.util.timber

import timber.log.Timber

class ReleaseTree : Timber.DebugTree() {
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
