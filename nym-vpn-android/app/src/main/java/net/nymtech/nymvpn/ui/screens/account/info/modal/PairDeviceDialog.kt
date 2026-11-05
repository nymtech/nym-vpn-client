package net.nymtech.nymvpn.ui.screens.account.info.modal

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.QrCode2
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.DialogProperties
import kotlinx.coroutines.delay
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.pairing.model.PairingState
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.OutlineStyledButton
import net.nymtech.nymvpn.ui.common.functions.SecureScreen
import net.nymtech.nymvpn.ui.common.security.SecureOverlayProtection
import net.nymtech.nymvpn.ui.screens.account.pairing.components.PairingPin
import net.nymtech.nymvpn.ui.screens.account.pairing.components.PairingStatus
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.messageRes
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.qrscanner.QrCodeImage
import java.util.Locale
import kotlin.time.Duration
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds
import kotlin.time.TimeMark
import kotlin.time.TimeSource

@Composable
fun PairDeviceDialog(state: PairingState, onNewCode: () -> Unit, onCodesMatch: () -> Unit, onDismiss: () -> Unit) {
	val remaining = (state as? PairingState.ShowingCode)?.let { rememberRemaining(it.expiresAt) }
	val expired = remaining != null && remaining <= Duration.ZERO
	val sessionLive = (state is PairingState.ShowingCode && !expired) || state is PairingState.ConfirmCode || state == PairingState.SharingAccount

	AlertDialog(
		containerColor = MaterialTheme.colorScheme.surface,
		tonalElevation = 0.dp,
		// An accidental tap outside would end the session the other device is joining.
		properties = DialogProperties(dismissOnClickOutside = !sessionLive),
		onDismissRequest = { if (state != PairingState.SharingAccount) onDismiss() },
		icon = {
			Box(
				modifier = Modifier
					.size(48.dp)
					.clip(RoundedCornerShape(12.dp))
					.background(MaterialTheme.colorScheme.primary.copy(alpha = 0.15f)),
				contentAlignment = Alignment.Center,
			) {
				Icon(
					imageVector = Icons.Outlined.QrCode2,
					contentDescription = null,
					tint = MaterialTheme.colorScheme.primary,
					modifier = Modifier.size(24.dp),
				)
			}
		},
		title = {
			Text(
				text = stringResource(R.string.pair_device_title),
				style = CustomTypography.titleMediumPlus,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				textAlign = TextAlign.Center,
			)
		},
		text = {
			SecureScreen()
			SecureOverlayProtection()
			Column(
				modifier = Modifier.fillMaxWidth(),
				horizontalAlignment = Alignment.CenterHorizontally,
				verticalArrangement = Arrangement.spacedBy(16.dp),
			) {
				when {
					expired -> PairingStatus(title = stringResource(R.string.pairing_expired))
					state is PairingState.ShowingCode && remaining != null -> ShowingCode(state, remaining)
					state is PairingState.ConfirmCode -> {
						PairingStatus(
							title = stringResource(R.string.pair_device_confirm_title),
							body = stringResource(R.string.pair_device_confirm_body),
						)
						PairingPin(pin = state.pin)
					}
					state == PairingState.SharingAccount -> PairingStatus(title = stringResource(R.string.pair_device_sharing), loading = true)
					state == PairingState.AccountShared -> PairingStatus(
						title = stringResource(R.string.pair_device_done_title),
						body = stringResource(R.string.pair_device_done_body),
					)
					state == PairingState.Expired -> PairingStatus(title = stringResource(R.string.pairing_expired))
					state is PairingState.Failed -> PairingStatus(title = stringResource(state.failure.messageRes()))
					else -> PairingStatus(title = stringResource(R.string.pair_device_starting), loading = true)
				}
			}
		},
		confirmButton = {
			Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
				when {
					expired || state == PairingState.Expired -> {
						DialogMainButton(stringResource(R.string.pair_device_new_code), onNewCode)
						DialogOutlineButton(stringResource(R.string.close), onDismiss)
					}
					state is PairingState.Failed -> {
						DialogMainButton(stringResource(R.string.pairing_try_again), onNewCode)
						DialogOutlineButton(stringResource(R.string.close), onDismiss)
					}
					state is PairingState.ConfirmCode -> {
						DialogMainButton(stringResource(R.string.pair_device_codes_match), onCodesMatch)
						DialogOutlineButton(stringResource(R.string.pair_device_codes_dont_match), onDismiss)
					}
					state == PairingState.AccountShared -> DialogMainButton(stringResource(R.string.pair_device_done), onDismiss)
					state == PairingState.SharingAccount -> Unit
					else -> DialogOutlineButton(stringResource(R.string.cancel), onDismiss)
				}
			}
		},
	)
}

@Composable
private fun ShowingCode(state: PairingState.ShowingCode, remaining: Duration) {
	Text(
		text = stringResource(R.string.pair_device_instructions),
		style = MaterialTheme.typography.bodyMedium,
		color = MaterialTheme.colorScheme.onSurface,
		textAlign = TextAlign.Center,
	)
	QrCodeImage(
		content = state.url,
		contentDescription = stringResource(R.string.pair_device_qr_description),
		modifier = Modifier
			.widthIn(max = 240.dp)
			.fillMaxWidth(),
	)
	Text(
		text = stringResource(R.string.pair_device_expires_in, remaining.formatMinutesSeconds()),
		style = MaterialTheme.typography.bodyMedium,
		color = MaterialTheme.colorScheme.outline,
	)
}

@Composable
private fun DialogMainButton(text: String, onClick: () -> Unit) {
	MainStyledButton(
		onClick = onClick,
		content = { Text(text, style = CustomTypography.buttonMain) },
		color = MaterialTheme.colorScheme.primary,
		modifier = Modifier
			.fillMaxWidth()
			.height(48.dp.scaledHeight()),
	)
}

@Composable
private fun DialogOutlineButton(text: String, onClick: () -> Unit) {
	OutlineStyledButton(
		onClick = onClick,
		content = {
			Text(
				text,
				style = MaterialTheme.typography.titleMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
		},
		modifier = Modifier
			.fillMaxWidth()
			.height(48.dp.scaledHeight()),
		shape = RoundedCornerShape(12.dp),
	)
}

@Composable
private fun rememberRemaining(expiresAt: TimeMark): Duration {
	var remaining by remember(expiresAt) { mutableStateOf(-expiresAt.elapsedNow()) }
	LaunchedEffect(expiresAt) {
		while (remaining > Duration.ZERO) {
			delay(1.seconds)
			remaining = -expiresAt.elapsedNow()
		}
	}
	return remaining
}

private fun Duration.formatMinutesSeconds(): String = toComponents { minutes, seconds, _ ->
	"%d:%02d".format(Locale.ROOT, minutes, seconds)
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun PreviewPairDeviceDialog() {
	NymVPNTheme(Theme.default()) {
		PairDeviceDialog(
			state = PairingState.ShowingCode(
				url = "https://nym.com/qr-pairing?pubkey=7EcDhSYGxXd4ndBwnrDJnLDkyHYv8BYM4A5MXqtWzzpm",
				sessionId = 1u,
				expiresAt = TimeSource.Monotonic.markNow() + 5.minutes,
			),
			onNewCode = {},
			onCodesMatch = {},
			onDismiss = {},
		)
	}
}
