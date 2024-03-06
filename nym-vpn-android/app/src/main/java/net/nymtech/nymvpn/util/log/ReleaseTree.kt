package net.nymtech.nymvpn.util.log

import android.util.Log
import io.sentry.Sentry
import timber.log.Timber

class ReleaseTree : Timber.Tree() {
    override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
        when(priority) {
            Log.DEBUG -> return
        }
        when(priority) {
            Log.ERROR -> if(t != null) {
                Sentry.captureException(t)
            } else {
                Sentry.captureException(NymAndroidException(message))
            }
        }
        super.log(priority, tag, message, t)
    }
}