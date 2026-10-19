package net.nymtech.nymvpn.manager.billing

import kotlinx.coroutines.flow.first
import kotlinx.coroutines.withTimeoutOrNull
import net.nymtech.billing.model.ProductData
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.util.Constants
import kotlin.time.Duration

fun BillingManager.isPlayBillingSupported(): Boolean = isAvailable() && BuildConfig.APPLICATION_ID == Constants.APP_ID

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
