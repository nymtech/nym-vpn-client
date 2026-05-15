package net.nymtech.nymvpn.ui.screens.settings.components

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.LocalNymColors

@Composable
fun SubscriptionStatusText(subscription: SubscriptionUiState?, modifier: Modifier = Modifier) {
	val color: Color
	val text: String

	when (subscription?.expiryState) {
		ExpiryState.NORMAL -> {
			color = MaterialTheme.colorScheme.tertiary
			text = stringResource(R.string.account_info_valid_text, subscription.validUntilDate)
		}

		ExpiryState.WARNING -> {
			color = LocalNymColors.current.warning
			text = stringResource(R.string.account_info_expires_text, subscription.validUntilDate)
		}

		ExpiryState.EXPIRED -> {
			color = MaterialTheme.colorScheme.error
			text = stringResource(R.string.account_info_no_plan)
		}

		ExpiryState.PENDING -> {
			color = MaterialTheme.colorScheme.error
			text = stringResource(R.string.account_info_confirming_payment)
		}

		else -> {
			color = MaterialTheme.colorScheme.error
			text = stringResource(R.string.account_info_no_plan)
		}
	}

	Text(
		text = text,
		style = MaterialTheme.typography.bodySmall.copy(color = color),
		modifier = modifier,
	)
}

enum class ExpiryState {
	NORMAL,
	WARNING,
	EXPIRED,
	PENDING,
}

data class SubscriptionUiState(val isRecurring: Boolean, val validUntilDate: String, val expiryState: ExpiryState = ExpiryState.NORMAL)
