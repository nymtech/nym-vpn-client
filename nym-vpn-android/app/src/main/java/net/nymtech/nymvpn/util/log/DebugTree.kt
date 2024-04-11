package net.nymtech.nymvpn.util.log

import io.sentry.Sentry
import timber.log.Timber

class DebugTree : Timber.DebugTree() {
	override fun e(t: Throwable?) {
		t?.let {
			Sentry.captureException(t)
		}
		super.e(t)
	}

	override fun e(message: String?, vararg args: Any?) {
		message?.let {
			Sentry.captureException(NymAndroidException(message))
		}
		super.e(message, *args)
	}
}
