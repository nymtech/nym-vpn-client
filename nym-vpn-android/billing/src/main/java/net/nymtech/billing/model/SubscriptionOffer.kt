package net.nymtech.billing.model

/**
 * A Play subscription offer reduced to what offer selection needs.
 */
data class SubscriptionOffer(val offerToken: String, val hasFreePhase: Boolean)

object SubscriptionOfferSelector {
	/**
	 * Picks the offer to launch for a subscription product.
	 *
	 * The trial offer (an offer whose pricing phases contain a free phase) is picked only when the
	 * backend said the account is eligible AND Play returned such an offer. Play omits the trial
	 * offer when the Google account already used one, in which case the account is treated as
	 * ineligible and the regular offer is launched. Returns null when no usable offer exists.
	 */
	fun select(offers: List<SubscriptionOffer>, trialEligible: Boolean): SubscriptionOffer? {
		val trial = offers.firstOrNull { it.hasFreePhase }
		val regular = offers.firstOrNull { !it.hasFreePhase }
		return when {
			trialEligible && trial != null -> trial
			else -> regular
		}
	}
}
