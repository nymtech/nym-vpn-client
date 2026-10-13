package net.nymtech.nymvpn.ui.screens.account.plan

import net.nymtech.billing.model.ProductData
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class SelectPlanUiStateTest {

	@Test
	fun benefitsWaitForEligibilityAndProducts() {
		assertFalse(SelectPlanUiState().benefitsResolved)
		assertFalse(SelectPlanUiState(productsResolved = true).benefitsResolved)
		assertFalse(SelectPlanUiState(lastResolvedTrialEligible = true).benefitsResolved)
		assertTrue(SelectPlanUiState(productsResolved = true, lastResolvedTrialEligible = false).benefitsResolved)
	}

	@Test
	fun eligibleWithPlayTrial_showsShortestTrial() {
		val state = resolved(eligible = true, product(14), product(null), product(7))

		assertEquals(7, state.benefitsTrialDays)
	}

	@Test
	fun eligibleWithoutPlayTrial_showsNoTrialRow() {
		assertNull(resolved(eligible = true, product(null), product(null)).benefitsTrialDays)
	}

	@Test
	fun notEligible_showsNoTrialRowEvenWithPlayTrial() {
		assertNull(resolved(eligible = false, product(7)).benefitsTrialDays)
	}

	@Test
	fun noProducts_showsNoTrialRow() {
		assertNull(resolved(eligible = true).benefitsTrialDays)
	}

	private fun resolved(eligible: Boolean, vararg products: ProductData) = SelectPlanUiState(
		subscriptions = products.toList(),
		productsResolved = true,
		lastResolvedTrialEligible = eligible,
	)

	private fun product(freeTrialDays: Int?) = object : ProductData {
		override val id = "id"
		override val name = "name"
		override val price = ""
		override val freeTrialDays = freeTrialDays
		override val priceAmountMicros = null
		override val priceCurrencyCode = null
	}
}
