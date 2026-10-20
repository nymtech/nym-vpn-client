package net.nymtech.nymvpn.ui.screens.onboarding.components

import net.nymtech.billing.model.ProductData
import net.nymtech.billing.model.ProductId
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class OnboardingPlanPricingTest {

	@Test
	fun eligibleWithPlayTrial_showsTrialFromPlay() {
		val pricing = OnboardingPlanPricing.resolve(listOf(monthly(), yearly(freeTrialDays = 3)), trialEligible = true)

		assertEquals(3, pricing.freeTrialDays)
	}

	@Test
	fun notEligibleWithPlayTrial_showsNoTrial() {
		val pricing = OnboardingPlanPricing.resolve(listOf(monthly(), yearly(freeTrialDays = 7)), trialEligible = false)

		assertNull(pricing.freeTrialDays)
	}

	@Test
	fun eligibleWithoutPlayTrial_showsNoTrial() {
		val pricing = OnboardingPlanPricing.resolve(listOf(monthly(), yearly(freeTrialDays = null)), trialEligible = true)

		assertNull(pricing.freeTrialDays)
	}

	@Test
	fun trialOnlyOnMonthly_showsNoTrial() {
		val pricing = OnboardingPlanPricing.resolve(listOf(monthly(freeTrialDays = 7), yearly(freeTrialDays = null)), trialEligible = true)

		assertNull(pricing.freeTrialDays)
	}

	@Test
	fun noPlayData_usesFallbackWithoutTrial() {
		val pricing = OnboardingPlanPricing.resolve(emptyList(), trialEligible = true)

		assertEquals(OnboardingPlanPricing.FALLBACK, pricing)
		assertNull(pricing.freeTrialDays)
	}

	@Test
	fun yearlyWithoutPrice_usesFallbackWithoutTrial() {
		val pricing = OnboardingPlanPricing.resolve(listOf(yearly(freeTrialDays = 7, priceAmountMicros = null)), trialEligible = true)

		assertEquals(OnboardingPlanPricing.FALLBACK, pricing)
		assertNull(pricing.freeTrialDays)
	}

	@Test
	fun fallbackNeverClaimsTrial() {
		assertNull(OnboardingPlanPricing.FALLBACK.freeTrialDays)
	}

	private fun monthly(freeTrialDays: Int? = null) = product(ProductId.Monthly, priceAmountMicros = 9_990_000, freeTrialDays = freeTrialDays)

	private fun yearly(freeTrialDays: Int?, priceAmountMicros: Long? = 59_990_000) = product(ProductId.Yearly, priceAmountMicros, freeTrialDays)

	private fun product(id: ProductId, priceAmountMicros: Long?, freeTrialDays: Int?) = object : ProductData {
		override val id = id.value
		override val name = id.name
		override val price = ""
		override val freeTrialDays = freeTrialDays
		override val priceAmountMicros = priceAmountMicros
		override val priceCurrencyCode = "USD"
	}
}
