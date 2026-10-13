package net.nymtech.billing.model

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class SubscriptionOfferSelectorTest {
	private val trial = SubscriptionOffer(offerToken = "trial", hasFreePhase = true)
	private val regular = SubscriptionOffer(offerToken = "regular", hasFreePhase = false)

	@Test
	fun eligibleAccountGetsTrialOffer() {
		assertEquals(trial, SubscriptionOfferSelector.select(listOf(regular, trial), trialEligible = true))
	}

	@Test
	fun ineligibleAccountGetsRegularOfferEvenWhenTrialIsListedFirst() {
		assertEquals(regular, SubscriptionOfferSelector.select(listOf(trial, regular), trialEligible = false))
	}

	@Test
	fun eligibleAccountWithoutTrialOfferFallsBackToRegularOffer() {
		assertEquals(regular, SubscriptionOfferSelector.select(listOf(regular), trialEligible = true))
	}

	@Test
	fun ineligibleAccountNeverGetsTrialOffer() {
		assertNull(SubscriptionOfferSelector.select(listOf(trial), trialEligible = false))
	}

	@Test
	fun noOffersSelectsNothing() {
		assertNull(SubscriptionOfferSelector.select(emptyList(), trialEligible = true))
	}
}
