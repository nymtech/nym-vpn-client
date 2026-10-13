package net.nymtech.billing.model

/** [offerId] is null for the base plan; trials and discounts are offers on top of it. */
data class SubscriptionOffer(val offerToken: String, val offerId: String?, val hasFreePhase: Boolean) {
	val isBasePlan: Boolean get() = offerId == null
	val isTrial: Boolean get() = offerId != null && hasFreePhase
}

object SubscriptionOfferSelector {
	/** Trial offer if eligible and Play returned one, else the base plan; null (no base plan) must be reported as a failure. */
	fun select(offers: List<SubscriptionOffer>, trialEligible: Boolean): SubscriptionOffer? {
		if (trialEligible) offers.firstOrNull { it.isTrial }?.let { return it }
		return offers.firstOrNull { it.isBasePlan }
	}
}
