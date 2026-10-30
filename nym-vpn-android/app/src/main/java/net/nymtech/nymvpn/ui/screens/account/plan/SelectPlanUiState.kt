package net.nymtech.nymvpn.ui.screens.account.plan

import net.nymtech.billing.model.ProductData
import net.nymtech.nymvpn.manager.billing.TrialEligibility

/** [lastResolvedTrialEligible] keeps the previous answer while the sheet re-reads eligibility, so the benefits list doesn't flicker. */
data class SelectPlanUiState(
	val subscriptions: List<ProductData> = emptyList(),
	val productsResolved: Boolean = false,
	val trialEligibility: TrialEligibility = TrialEligibility.Loading,
	val lastResolvedTrialEligible: Boolean? = null,
	val isConnectingBilling: Boolean = false,
) {
	val benefitsResolved: Boolean get() = productsResolved && lastResolvedTrialEligible != null

	/** Shortest Play trial when eligible, so the label never overpromises; null hides the row. */
	val benefitsTrialDays: Int?
		get() = if (lastResolvedTrialEligible == true) subscriptions.mapNotNull { it.freeTrialDays }.minOrNull() else null
}
