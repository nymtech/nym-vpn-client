package net.nymtech.nymvpn.util.logs.timber

import timber.log.Timber

class NoLogTree : Timber.Tree() {
	override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
		// Intentionally empty: drops all logs
	}
}
