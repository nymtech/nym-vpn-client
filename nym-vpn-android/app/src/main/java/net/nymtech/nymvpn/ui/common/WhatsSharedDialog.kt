package net.nymtech.nymvpn.ui.common

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.util.extensions.openWebUrl

/** "What's shared" popup for the telemetry opt-in toggles, shared by the onboarding
 *  opt-in screen and the Data & privacy settings screen. A Dialog (not a bottom
 *  sheet) so it layers cleanly over the onboarding bottom sheet. */
@Composable
fun WhatsSharedDialog(kind: TelemetryInfoKind, onDismiss: () -> Unit) {
	val context = LocalContext.current

	val sharesRes = when (kind) {
		TelemetryInfoKind.CONNECTION -> R.string.optin_connection_shares
		TelemetryInfoKind.MONITORING -> R.string.optin_monitoring_shares
	}
	val neverRes = when (kind) {
		TelemetryInfoKind.CONNECTION -> R.string.optin_connection_never
		TelemetryInfoKind.MONITORING -> R.string.optin_monitoring_never
	}
	val handlingRes = when (kind) {
		TelemetryInfoKind.CONNECTION -> R.string.optin_connection_handling
		TelemetryInfoKind.MONITORING -> R.string.optin_monitoring_handling
	}
	val bullets = stringResource(sharesRes).split("\n")

	Dialog(onDismissRequest = onDismiss) {
		Surface(
			shape = RoundedCornerShape(16.dp),
			color = MaterialTheme.colorScheme.surface,
		) {
			Column(
				modifier = Modifier
					.fillMaxWidth()
					.padding(24.dp),
				verticalArrangement = Arrangement.spacedBy(12.dp),
			) {
				Icon(
					Icons.Outlined.Info,
					contentDescription = null,
					tint = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier.align(Alignment.CenterHorizontally),
				)
				Text(
					text = stringResource(R.string.optin_info_title),
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier.align(Alignment.CenterHorizontally),
				)

				Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
					bullets.forEach { bullet ->
						Text(
							text = "•  $bullet",
							style = MaterialTheme.typography.bodyMedium,
							color = MaterialTheme.colorScheme.onBackground,
						)
					}
				}

				HorizontalDivider()

				Text(
					text = buildAnnotatedString {
						withStyle(SpanStyle(fontWeight = FontWeight.SemiBold)) {
							append(stringResource(R.string.optin_info_never_label))
						}
						append(" ")
						append(stringResource(neverRes))
					},
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.onBackground,
				)
				Text(
					text = stringResource(handlingRes),
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.onBackground,
				)

				MainStyledButton(
					onClick = onDismiss,
					content = {
						Text(
							text = stringResource(R.string.optin_info_got_it),
							style = MaterialTheme.typography.titleMedium,
						)
					},
					color = MaterialTheme.colorScheme.primary,
					modifier = Modifier.fillMaxWidth(),
					shape = RoundedCornerShape(12.dp),
				)
				Text(
					text = stringResource(R.string.optin_privacy_link_text),
					style = MaterialTheme.typography.bodyMedium.copy(
						textDecoration = TextDecoration.Underline,
					),
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier
						.align(Alignment.CenterHorizontally)
						.clickable {
							context.openWebUrl(context.getString(R.string.optin_privacy_link))
						},
				)
			}
		}
	}
}
