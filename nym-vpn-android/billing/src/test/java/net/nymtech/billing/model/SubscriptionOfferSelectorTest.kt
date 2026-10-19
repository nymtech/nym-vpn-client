package net.nymtech.billing.model

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class SubscriptionOfferSelectorTest {
	private val basePlan = SubscriptionOffer(offerToken = "base", offerId = null, hasFreePhase = false)
	private val trial = SubscriptionOffer(offerToken = "trial", offerId = "free-trial", hasFreePhase = true)
	private val discount = SubscriptionOffer(offerToken = "discount", offerId = "intro-discount", hasFreePhase = false)

	@Test
	fun eligibleAccountGetsTrialOffer() {
		assertEquals(trial, SubscriptionOfferSelector.select(listOf(basePlan, trial), trialEligible = true))
	}

	@Test
	fun ineligibleAccountGetsBasePlanEvenWhenTrialIsListedFirst() {
		assertEquals(basePlan, SubscriptionOfferSelector.select(listOf(trial, basePlan), trialEligible = false))
	}

	@Test
	fun eligibleAccountWithoutTrialOfferGetsBasePlan() {
		assertEquals(basePlan, SubscriptionOfferSelector.select(listOf(basePlan), trialEligible = true))
	}

	@Test
	fun basePlanIsChosenOverNonFreeDiscountOffer() {
		assertEquals(basePlan, SubscriptionOfferSelector.select(listOf(discount, basePlan), trialEligible = false))
		assertEquals(basePlan, SubscriptionOfferSelector.select(listOf(discount, basePlan), trialEligible = true))
	}

	@Test
	fun ineligibleAccountNeverGetsTrialOfferEvenWhenItIsTheOnlyOffer() {
		assertNull(SubscriptionOfferSelector.select(listOf(trial), trialEligible = false))
	}

	@Test
	fun discountOfferIsNeverUsedAsBasePlanFallback() {
		assertNull(SubscriptionOfferSelector.select(listOf(discount), trialEligible = false))
	}

	@Test
	fun noOffersSelectsNothing() {
		assertNull(SubscriptionOfferSelector.select(emptyList(), trialEligible = true))
		assertNull(SubscriptionOfferSelector.select(emptyList(), trialEligible = false))
	}
}
