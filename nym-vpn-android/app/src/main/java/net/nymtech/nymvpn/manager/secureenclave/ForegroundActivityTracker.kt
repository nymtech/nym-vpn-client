package net.nymtech.nymvpn.manager.secureenclave

import android.app.Activity
import android.app.Application
import android.os.Bundle
import androidx.fragment.app.FragmentActivity
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Keeps track of the activity that is currently resumed, if any.
 */
@Singleton
class ForegroundActivityTracker @Inject constructor() : Application.ActivityLifecycleCallbacks {

	@Volatile
	var resumedActivity: FragmentActivity? = null
		private set

	override fun onActivityResumed(activity: Activity) {
		resumedActivity = activity as? FragmentActivity
	}

	override fun onActivityPaused(activity: Activity) {
		if (resumedActivity === activity) resumedActivity = null
	}

	override fun onActivityCreated(activity: Activity, savedInstanceState: Bundle?) = Unit
	override fun onActivityStarted(activity: Activity) = Unit
	override fun onActivityStopped(activity: Activity) = Unit
	override fun onActivitySaveInstanceState(activity: Activity, outState: Bundle) = Unit
	override fun onActivityDestroyed(activity: Activity) = Unit
}
