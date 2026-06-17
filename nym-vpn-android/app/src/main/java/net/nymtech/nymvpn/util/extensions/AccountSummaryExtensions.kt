package net.nymtech.nymvpn.util.extensions

import net.nymtech.nymvpn.ui.screens.account.info.components.BandwidthUiState
import net.nymtech.nymvpn.ui.screens.settings.components.ExpiryState
import net.nymtech.nymvpn.ui.screens.settings.components.SubscriptionUiState
import nym_vpn_lib_types.NymVpnSubscriptionKind
import nym_vpn_lib_types.NymVpnSubscriptionStatus
import nym_vpn_lib_types.VpnAccountSummary
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.ZoneOffset
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import java.time.temporal.ChronoUnit

/**
 * Converts a Unix timestamp (Long) to ZonedDateTime in the system's local timezone.
 */
private fun Long.toZonedDateTime(): ZonedDateTime {
	val instant = if (this > 1000000000000L) {
		Instant.ofEpochMilli(this)
	} else {
		Instant.ofEpochSecond(this)
	}
	return instant.atZone(ZoneId.systemDefault())
}

/**
 * Converts a Unix timestamp (Long) to ZonedDateTime in UTC.
 */
private fun Long.toUtcZonedDateTime(): ZonedDateTime {
	val instant = if (this > 1000000000000L) {
		Instant.ofEpochMilli(this)
	} else {
		Instant.ofEpochSecond(this)
	}
	return instant.atZone(ZoneOffset.UTC)
}

private fun calculateExpiryState(isRecurring: Boolean, isActive: Boolean, planType: NymVpnSubscriptionKind?, expiryTimestamp: Long?): ExpiryState {
	if (!isActive) return ExpiryState.EXPIRED
	if (expiryTimestamp == null) return ExpiryState.NORMAL

	if (isRecurring) return ExpiryState.NORMAL

	val today = LocalDate.now(ZoneId.systemDefault())
	val expiryDate = expiryTimestamp.toZonedDateTime().toLocalDate()
	val daysUntilExpiry = ChronoUnit.DAYS.between(today, expiryDate)

	if (daysUntilExpiry < 0) return ExpiryState.EXPIRED

	return when (planType) {
		is NymVpnSubscriptionKind.OneMonth -> when {
			daysUntilExpiry < 7 -> ExpiryState.WARNING
			else -> ExpiryState.NORMAL
		}
		is NymVpnSubscriptionKind.OneYear,
		is NymVpnSubscriptionKind.TwoYears,
		-> when {
			daysUntilExpiry < 15 -> ExpiryState.WARNING
			else -> ExpiryState.NORMAL
		}
		else -> ExpiryState.NORMAL
	}
}

fun VpnAccountSummary.toSubscriptionUiState(): SubscriptionUiState {
	val subscriptionWrapper = this.subscription
	val innerSubscription = subscriptionWrapper?.subscription
	val isRecurring = innerSubscription?.isRecurring ?: false
	val validUntilUtc = innerSubscription?.validUntilUtc

	if (subscriptionWrapper?.status == NymVpnSubscriptionStatus.PENDING) {
		return SubscriptionUiState(
			isRecurring = isRecurring,
			validUntilDate = "",
			expiryState = ExpiryState.PENDING,
		)
	}

	val expiryFormatter = DateTimeFormatter.ofPattern("d MMMM yyyy")
	val expiryDateStr = validUntilUtc?.toZonedDateTime()?.format(expiryFormatter) ?: "Unknown"

	val expiryState = calculateExpiryState(
		isRecurring = isRecurring,
		isActive = this.isSubscriptionActive(),
		planType = innerSubscription?.kind,
		expiryTimestamp = validUntilUtc,
	)

	return SubscriptionUiState(
		isRecurring = isRecurring,
		validUntilDate = expiryDateStr,
		expiryState = expiryState,
	)
}

fun VpnAccountSummary.toBandwidthUiState(): BandwidthUiState {
	val resetTimeStr = this.trafficResetTime
		?.toUtcZonedDateTime()
		?.format(DateTimeFormatter.ofPattern("HH:mm"))
		?.let { "$it UTC" } ?: "—"

	if (this.fairUsageDataUnavailable) {
		return BandwidthUiState(
			consumedGb = 0f,
			totalGb = 0f,
			percentage = 0f,
			resetDate = resetTimeStr,
			isUnavailable = true,
		)
	}

	val consumedGb = this.trafficUsedGb.toFloat()
	val totalGb = this.trafficLimitGb.toFloat()
	val bandwidthPercentage = if (totalGb > 0f) {
		(consumedGb / totalGb).coerceIn(0f, 1f)
	} else {
		0f
	}

	return BandwidthUiState(
		consumedGb = consumedGb,
		totalGb = totalGb,
		percentage = bandwidthPercentage,
		resetDate = resetTimeStr,
		isUnavailable = false,
	)
}
