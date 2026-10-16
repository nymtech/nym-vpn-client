package net.nymtech.nymvpn.manager.billing

sealed interface TrialEligibility {
	data object Loading : TrialEligibility

	data class Resolved(val eligible: Boolean) : TrialEligibility
}

val TrialEligibility.isEligible: Boolean get() = (this as? TrialEligibility.Resolved)?.eligible == true
