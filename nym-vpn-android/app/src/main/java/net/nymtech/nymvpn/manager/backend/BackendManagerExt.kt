package net.nymtech.nymvpn.manager.backend

import timber.log.Timber

suspend fun BackendManager.hasValidSubscription(tag: String = "BackendManagerExt"): Boolean {
	return runCatching {
		val summary = this.getAccountSummary()
		if (summary == null) {
			Timber.tag(tag).w("AccountSummaryNull, treating as no subscription")
			return false
		}
		val isActive = summary.isSubscriptionActive()
		Timber.tag(tag).i("SubscriptionCheck active=%s", isActive)
		isActive
	}.getOrElse { t ->
		Timber.tag(tag).e(t, "AccountSummaryFetchFailed")
		false
	}
}
