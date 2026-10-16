package net.nymtech.nymvpn.ui.screens.onboarding.components

import net.nymtech.billing.model.ProductData
import net.nymtech.billing.model.ProductId
import java.math.BigDecimal
import java.math.RoundingMode
import java.text.NumberFormat
import java.util.Currency

/** [freeTrialDays] is set only when the account is eligible AND Play offers a yearly trial. */
data class OnboardingPlanPricing(val monthlyEquivalentPrice: String, val savingsPercent: String?, val freeTrialDays: Int?) {
	companion object {
		/** No trial: without Play data eligibility can't be matched to an offer. */
		val FALLBACK = OnboardingPlanPricing(monthlyEquivalentPrice = "$8.26", savingsPercent = "65%", freeTrialDays = null)

		fun resolve(products: List<ProductData>, trialEligible: Boolean): OnboardingPlanPricing = from(products, trialEligible) ?: FALLBACK

		fun from(products: List<ProductData>, trialEligible: Boolean): OnboardingPlanPricing? {
			val yearly = products.firstOrNull { it.id == ProductId.Yearly.value } ?: return null
			val yearlyMicros = yearly.priceAmountMicros ?: return null
			val currencyCode = yearly.priceCurrencyCode ?: return null
			val formatter = runCatching {
				val currency = Currency.getInstance(currencyCode)
				NumberFormat.getCurrencyInstance().apply {
					this.currency = currency
					maximumFractionDigits = currency.defaultFractionDigits
					minimumFractionDigits = currency.defaultFractionDigits
				}
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
				freeTrialDays = yearly.freeTrialDays.takeIf { trialEligible },
			)
		}

		private val MICROS_PER_UNIT = BigDecimal(1_000_000)
	}
}
