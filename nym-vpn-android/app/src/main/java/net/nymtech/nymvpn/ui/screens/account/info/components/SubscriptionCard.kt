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
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.Launch
import androidx.compose.material.icons.outlined.Bolt
import androidx.compose.material.icons.outlined.GppBad
import androidx.compose.material.icons.outlined.Speed
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.screens.settings.components.ExpiryState
import net.nymtech.nymvpn.ui.screens.settings.components.SubscriptionUiState
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun SubscriptionSection(subscriptionState: SubscriptionUiState?, bandwidthState: BandwidthUiState?, onSelectPlanClick: () -> Unit, onRenewClick: () -> Unit, onContactSupportClick: () -> Unit) {
	if (subscriptionState == null || bandwidthState == null) return

	Column(modifier = Modifier.fillMaxWidth()) {
		if (subscriptionState.expiryState == ExpiryState.EXPIRED) {
			MainStyledButton(
				onClick = onSelectPlanClick,
				content = {
					Text(
						stringResource(R.string.select_plan_button),
						style = CustomTypography.buttonMain,
					)
				},
				color = MaterialTheme.colorScheme.primary,
				modifier = Modifier
					.fillMaxWidth()
					.height(56.dp.scaledHeight()),
			)

			Spacer(modifier = Modifier.height(16.dp))
			ExpiredCard()
		} else {
			SubscriptionCard(
				subscription = subscriptionState,
				bandwidth = bandwidthState,
				onRenewClick = onRenewClick,
			)

			Spacer(modifier = Modifier.height(16.dp))
			ContactSupportText(onClick = onContactSupportClick)
		}
	}
}

@Composable
private fun BaseAccountStatusCard(content: @Composable ColumnScope.() -> Unit) {
	Card(
		modifier = Modifier.fillMaxWidth(),
		shape = RoundedCornerShape(8.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
	) {
		Column(modifier = Modifier.fillMaxWidth()) {
			Row(
				modifier = Modifier.padding(16.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				Icon(Icons.Outlined.Speed, contentDescription = null, tint = MaterialTheme.colorScheme.outline)
				Spacer(Modifier.width(12.dp))
				Text(
					text = stringResource(R.string.account_info_status_title),
					style = MaterialTheme.typography.titleMedium.copy(color = MaterialTheme.colorScheme.onSurface),
				)
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
					style = MaterialTheme.typography.bodySmall.copy(color = MaterialTheme.colorScheme.primary),
				)
				Text(
					text = stringResource(R.string.account_info_limit_text),
					style = MaterialTheme.typography.bodySmall.copy(color = MaterialTheme.colorScheme.outline),
				)
			}

			Spacer(Modifier.height(8.dp))

			val remainingGb = maxOf(0f, bandwidth.totalGb - bandwidth.consumedGb)
			val progress = if (bandwidth.totalGb > 0) remainingGb / bandwidth.totalGb else 0f

			LinearProgressIndicator(
				progress = { progress },
				modifier = Modifier
					.fillMaxWidth()
					.height(6.dp),
				color = MaterialTheme.colorScheme.primary,
				trackColor = MaterialTheme.colorScheme.background,
				strokeCap = StrokeCap.Round,
			)

			Spacer(Modifier.height(8.dp))

			Row(
				modifier = Modifier.fillMaxWidth(),
				horizontalArrangement = Arrangement.SpaceBetween,
			) {
				val format = java.text.NumberFormat.getNumberInstance(java.util.Locale.US)
				Text(
					text = "${format.format(remainingGb.toInt())} GB",
					style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.primary),
				)
				Text(
					text = "${format.format(bandwidth.totalGb.toInt())} GB",
					style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
				)
			}
		}

		Spacer(Modifier.height(24.dp))
		HorizontalDivider(color = MaterialTheme.colorScheme.background)

		Row(
			modifier = Modifier
				.fillMaxWidth()
				.padding(16.dp),
			horizontalArrangement = Arrangement.SpaceBetween,
		) {
			Text(
				text = stringResource(R.string.account_info_reset_text),
				style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.outline),
			)
			Text(
				text = bandwidth.resetDate,
				style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
			)
		}

		if (subscription.expiryState == ExpiryState.WARNING_YELLOW || subscription.expiryState == ExpiryState.WARNING_AMBER) {
			val isAmber = subscription.expiryState == ExpiryState.WARNING_AMBER
			val contentColor = if (isAmber) CustomColors.warning else MaterialTheme.colorScheme.primary
			val bgColor = if (isAmber) CustomColors.warning.copy(alpha = 0.1f) else CustomColors.statusGreen

			Row(
				modifier = Modifier
					.fillMaxWidth()
					.background(bgColor)
					.clickable { onRenewClick() }
					.padding(horizontal = 16.dp, vertical = 20.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				Icon(Icons.Outlined.Bolt, contentDescription = null, tint = contentColor)
				Spacer(Modifier.width(12.dp))
				Text(
					text = stringResource(R.string.account_info_renew_text),
					style = MaterialTheme.typography.bodyMedium.copy(color = contentColor),
					modifier = Modifier.weight(1f),
				)
				Icon(Icons.AutoMirrored.Outlined.Launch, contentDescription = null, tint = contentColor)
			}
		}
	}
}

@Composable
private fun ExpiredCard() {
	BaseAccountStatusCard {
		HorizontalDivider(color = MaterialTheme.colorScheme.background)

		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(vertical = 40.dp),
			horizontalAlignment = Alignment.CenterHorizontally,
		) {
			Box(
				modifier = Modifier
					.size(64.dp)
					.border(1.dp, MaterialTheme.colorScheme.outline, CircleShape),
				contentAlignment = Alignment.Center,
			) {
				Icon(
					imageVector = Icons.Outlined.GppBad,
					contentDescription = null,
					tint = MaterialTheme.colorScheme.outline,
					modifier = Modifier.size(32.dp),
				)
			}
			Spacer(Modifier.height(16.dp))
			Text(
				text = stringResource(R.string.account_info_no_plan),
				style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
			)
		}
	}
}

@Composable
private fun ContactSupportText(onClick: () -> Unit) {
	val annotatedString = buildAnnotatedString {
		withStyle(SpanStyle(color = CustomColors.warning)) {
			append(stringResource(R.string.account_info_contact_support_icon))
		}
		withStyle(
			SpanStyle(
				color = MaterialTheme.colorScheme.onSurface,
				textDecoration = TextDecoration.Underline,
			),
		) {
			append(stringResource(R.string.account_info_contact_support_action))
		}
		withStyle(SpanStyle(color = MaterialTheme.colorScheme.outline)) {
			append(stringResource(R.string.account_info_contact_support_suffix))
		}
	}

	Text(
		text = annotatedString,
		style = MaterialTheme.typography.bodyMedium,
		modifier = Modifier
			.fillMaxWidth()
			.clickable { onClick() }
			.padding(horizontal = 8.dp, vertical = 8.dp),
	)
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
