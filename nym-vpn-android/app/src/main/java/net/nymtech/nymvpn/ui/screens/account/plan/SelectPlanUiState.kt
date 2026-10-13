package net.nymtech.nymvpn.ui.screens.account.plan

import net.nymtech.billing.model.ProductData

/**
 * [trialEligible] is false until the backend explicitly confirms eligibility, so the trial label
 * is never shown for an account that would be charged the full price.
 *
 * [trialEligibilityResolved] is false while eligibility is being (re)read. Plans can't be selected
 * until it is true, so the launched offer always uses a fresh answer.
 */
data class SelectPlanUiState(
	val subscriptions: List<ProductData> = emptyList(),
	val trialEligible: Boolean = false,
	val trialEligibilityResolved: Boolean = false,
)
