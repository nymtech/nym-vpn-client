package net.nymtech.nymvpn.ui.screens.onboarding.components

import net.nymtech.billing.model.ProductData
import net.nymtech.billing.model.ProductId
import java.math.BigDecimal
import java.math.RoundingMode
import java.text.NumberFormat
import java.util.Currency

data class OnboardingPlanPricing(val monthlyEquivalentPrice: String, val savingsPercent: String?, val freeTrialDays: Int?) {
	companion object {
		fun from(products: List<ProductData>): OnboardingPlanPricing? {
			val yearly = products.firstOrNull { it.id == ProductId.Yearly.value } ?: return null
			val yearlyMicros = yearly.priceAmountMicros ?: return null
			val currencyCode = yearly.priceCurrencyCode ?: return null
			val formatter = runCatching {
				NumberFormat.getCurrencyInstance().apply { currency = Currency.getInstance(currencyCode) }
			}.getOrNull() ?: return null

			val yearlyAmount = BigDecimal(yearlyMicros).divide(MICROS_PER_UNIT)
			val monthlyEquivalent = yearlyAmount.divide(BigDecimal(12), 2, RoundingMode.HALF_UP)

			val monthly = products.firstOrNull { it.id == ProductId.Monthly.value }
			val savingsPercent = monthly?.priceAmountMicros?.let { monthlyMicros ->
				val yearOfMonthlyPayments = BigDecimal(monthlyMicros).divide(MICROS_PER_UNIT).multiply(BigDecimal(12))
				if (yearOfMonthlyPayments <= BigDecimal.ZERO || yearlyAmount >= yearOfMonthlyPayments) return@let null
				val saved = yearOfMonthlyPayments.subtract(yearlyAmount).divide(yearOfMonthlyPayments, 4, RoundingMode.HALF_UP)
				"${saved.multiply(BigDecimal(100)).setScale(0, RoundingMode.HALF_UP)}%"
			}

			return OnboardingPlanPricing(
				monthlyEquivalentPrice = formatter.format(monthlyEquivalent),
				savingsPercent = savingsPercent,
				freeTrialDays = yearly.freeTrialDays,
			)
		}

		private val MICROS_PER_UNIT = BigDecimal(1_000_000)
	}
}
