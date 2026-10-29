package net.nymtech.nymvpn.manager.billing

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.withTimeoutOrNull
import net.nymtech.billing.model.ProductData
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.util.Constants
import kotlin.time.Duration

fun BillingManager.isPlayBillingSupported(): Boolean = isAvailable() && BuildConfig.APPLICATION_ID == Constants.APP_ID

/**
 * Builds distributed through Google Play. Play's Payments policy forbids leading these users to
 * web checkout, so plan selection, renewal and management must stay on Play billing.
 */
val isPlayDistribution: Boolean get() = BuildConfig.FLAVOR == "general"

/** Google Play's subscription center for this app; lists only subscriptions bought through Play. */
const val PLAY_SUBSCRIPTIONS_URL = "https://play.google.com/store/account/subscriptions?package=${Constants.APP_ID}"

/** (Re)connects to Play billing and waits until it is ready; false if unsupported or [timeout] passes. */
suspend fun BillingManager.awaitReady(timeout: Duration): Boolean {
	if (!isPlayBillingSupported()) return false
	if (isReady()) return true
	initialize()
	return withTimeoutOrNull(timeout) {
		while (!isReady()) delay(BILLING_READY_POLL_MS)
		true
	} ?: false
}

private const val BILLING_READY_POLL_MS = 200L

/** First product list from Play, or empty when billing is unsupported or [timeout] passes. */
suspend fun BillingManager.awaitProducts(timeout: Duration): List<ProductData> {
	if (!isPlayBillingSupported()) return emptyList()
	return withTimeoutOrNull(timeout) {
		initialize()
		uiState.first { isReady() }
		fetchSubscriptions()
		products.first()
	} ?: emptyList()
}
