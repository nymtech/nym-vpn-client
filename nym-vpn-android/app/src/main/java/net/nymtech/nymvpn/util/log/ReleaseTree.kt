package net.nymtech.nymvpn.util.log

import io.sentry.Sentry
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
