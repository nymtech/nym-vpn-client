package net.nymtech.nymvpn.ui.screens.account.info.components

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.draw.clip
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.Launch
import androidx.compose.material.icons.outlined.AccessTime
import androidx.compose.material.icons.outlined.Bolt
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.screens.settings.components.ExpiryState
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsIcon
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsTitle
import net.nymtech.nymvpn.ui.screens.settings.components.SubscriptionUiState
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun SubscriptionSection(subscriptionState: SubscriptionUiState?, bandwidthState: BandwidthUiState?, onSelectPlanClick: () -> Unit, onRenewClick: () -> Unit, onContactSupportClick: () -> Unit) {
	Column(modifier = Modifier.fillMaxWidth()) {
		when {
			subscriptionState == null || subscriptionState.expiryState == ExpiryState.EXPIRED -> {
				MainStyledButton(
					onClick = onSelectPlanClick,
					content = {
						Text(
							stringResource(R.string.select_plan_button),
							style = MaterialTheme.typography.titleMedium,
						)
					},
					modifier = Modifier
						.fillMaxWidth()
						.height(48.dp.scaledHeight()),
					shape = RoundedCornerShape(12.dp),
				)

				Spacer(modifier = Modifier.height(16.dp))
				ExpiredCard(onRenewClick)
			}

			subscriptionState?.expiryState == ExpiryState.PENDING -> {
				ExpiredCard(onRenewClick)
			}
			else -> {
				if (bandwidthState != null) {
					SubscriptionCard(
						subscription = subscriptionState,
						bandwidth = bandwidthState,
						onRenewClick = onRenewClick,
					)

					Spacer(modifier = Modifier.height(16.dp))
				}
			}
		}
	}
}

@Composable
private fun BaseAccountStatusCard(content: @Composable ColumnScope.() -> Unit) {
	Card(
		modifier = Modifier.fillMaxWidth(),
		shape = RoundedCornerShape(8.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
	) {
		Column(modifier = Modifier.fillMaxWidth()) {
			Row(
				modifier = Modifier.padding(16.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				SettingsIcon(Icons.Outlined.AccessTime, "")
				Spacer(Modifier.width(18.dp))
				SettingsTitle(stringResource(R.string.account_info_status_title))
			}
			content()
		}
	}
}

@Composable
private fun SubscriptionCard(subscription: SubscriptionUiState, bandwidth: BandwidthUiState, onRenewClick: () -> Unit) {
	BaseAccountStatusCard {
		Column(modifier = Modifier.padding(horizontal = 16.dp)) {
			Row(
				modifier = Modifier.fillMaxWidth(),
				horizontalArrangement = Arrangement.SpaceBetween,
			) {
				Text(
					text = stringResource(R.string.account_info_bandwidth_title),
					style = MaterialTheme.typography.bodySmall.copy(color = MaterialTheme.colorScheme.tertiary),
				)
				Text(
					text = stringResource(R.string.account_info_limit_text),
					style = MaterialTheme.typography.bodySmall.copy(color = MaterialTheme.colorScheme.outline),
				)
			}

			Spacer(Modifier.height(8.dp))

			val remainingGb = maxOf(0f, bandwidth.totalGb - bandwidth.consumedGb)
			val progress = if (bandwidth.totalGb > 0) remainingGb / bandwidth.totalGb else 0f

			Box(
				modifier = Modifier
					.fillMaxWidth()
					.height(6.dp)
					.clip(RoundedCornerShape(50))
					.background(MaterialTheme.colorScheme.background),
			) {
				Box(
					modifier = Modifier
						.fillMaxWidth(progress)
						.fillMaxHeight()
						.background(MaterialTheme.colorScheme.tertiary),
				)
			}

			Spacer(Modifier.height(8.dp))

			Row(
				modifier = Modifier.fillMaxWidth(),
				horizontalArrangement = Arrangement.SpaceBetween,
			) {
				val format = java.text.NumberFormat.getNumberInstance(java.util.Locale.US)
				Text(
					text = "${format.format(remainingGb.toInt())} GB",
					style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onPrimaryContainer),
				)
				Text(
					text = "${format.format(bandwidth.totalGb.toInt())} GB",
					style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
				)
			}
		}

		Spacer(Modifier.height(24.dp))
		HorizontalDivider(color = MaterialTheme.colorScheme.surfaceVariant)

		Row(
			modifier = Modifier
				.fillMaxWidth()
				.padding(16.dp),
			horizontalArrangement = Arrangement.SpaceBetween,
		) {
			Text(
				stringResource(R.string.account_info_reset_text),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onSurfaceVariant,
			)
			Text(
				bandwidth.resetDate,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
		}

		if (subscription.expiryState == ExpiryState.WARNING) {
			HorizontalDivider(color = MaterialTheme.colorScheme.surfaceVariant)
			Row(
				modifier = Modifier
					.fillMaxWidth()
					.clickable { onRenewClick() }
					.padding(horizontal = 16.dp, vertical = 12.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				Icon(Icons.Outlined.Bolt, contentDescription = null, tint = LocalNymColors.current.warning)
				Spacer(Modifier.width(12.dp))
				Text(
					text = stringResource(R.string.account_info_renew_text),
					style = MaterialTheme.typography.bodySmall,
					color = LocalNymColors.current.warning,
					modifier = Modifier.weight(1f),
				)
				Icon(Icons.AutoMirrored.Outlined.Launch, contentDescription = null, tint = LocalNymColors.current.warning, modifier = Modifier.size(14.dp))
			}
		}
	}
}

@Composable
private fun ExpiredCard(onRenewClick: () -> Unit) {
	BaseAccountStatusCard {
		HorizontalDivider(color = MaterialTheme.colorScheme.surfaceVariant)

		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(vertical = 24.dp),
			horizontalAlignment = Alignment.CenterHorizontally,
		) {
			Box(
				modifier = Modifier
					.size(54.dp)
					.background(LocalNymColors.current.buttonErrorBorder.copy(alpha = 0.3F), CircleShape)
					.border(1.dp, LocalNymColors.current.buttonErrorBorder, CircleShape),
				contentAlignment = Alignment.Center,
			) {
				Icon(
					imageVector = ImageVector.vectorResource(R.drawable.ic_close_circle),
					contentDescription = null,
					tint = MaterialTheme.colorScheme.error,
					modifier = Modifier.size(32.dp),
				)
			}
			Spacer(Modifier.height(16.dp))
			Text(
				text = stringResource(R.string.account_info_no_plan),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.error,
			)
		}
		HorizontalDivider(color = MaterialTheme.colorScheme.surfaceVariant)
		Row(
			modifier = Modifier
				.fillMaxWidth()
				.clickable { onRenewClick() }
				.padding(horizontal = 16.dp, vertical = 12.dp),
			verticalAlignment = Alignment.CenterVertically,
		) {
			Icon(Icons.Outlined.Bolt, contentDescription = null, tint = MaterialTheme.colorScheme.error)
			Spacer(Modifier.width(12.dp))
			Text(
				text = stringResource(R.string.account_info_renew_text),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.error,
				modifier = Modifier.weight(1f),
			)
			Icon(Icons.AutoMirrored.Outlined.Launch, contentDescription = null, tint = MaterialTheme.colorScheme.error, modifier = Modifier.size(14.dp))
		}
	}
}

data class BandwidthUiState(val consumedGb: Float, val totalGb: Float, val percentage: Float, val resetDate: String)

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun PreviewAccountStatusNormal() {
	NymVPNTheme(Theme.default()) {
		Scaffold { padding ->
			Box(modifier = Modifier.padding(padding).padding(16.dp)) {
				SubscriptionSection(
					subscriptionState = SubscriptionUiState(
						isRecurring = true,
						validUntilDate = "December 24, 2026",
						expiryState = ExpiryState.NORMAL,
					),
					bandwidthState = BandwidthUiState(
						consumedGb = 800f,
						totalGb = 2000f,
						percentage = 0.4f,
						resetDate = "2026.03.18",
					),
					onSelectPlanClick = {},
					onRenewClick = {},
					onContactSupportClick = {},
				)
			}
		}
	}
}
