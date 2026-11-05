package net.nymtech.nymvpn.ui.screens.account.pairing

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.delay
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.pairing.model.PairingState
import net.nymtech.nymvpn.ui.common.buttons.OutlineStyledButton
import net.nymtech.nymvpn.ui.common.deviceauth.DeviceAuthAction
import net.nymtech.nymvpn.ui.common.deviceauth.rememberDeviceAuth
import net.nymtech.nymvpn.ui.common.functions.SecureScreen
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.security.SecureOverlayProtection
import net.nymtech.nymvpn.ui.screens.account.pairing.components.PairingButton
import net.nymtech.nymvpn.ui.screens.account.pairing.components.PairingPin
import net.nymtech.nymvpn.ui.screens.account.pairing.components.PairingStatus
import net.nymtech.nymvpn.util.extensions.messageRes
import net.nymtech.nymvpn.util.extensions.safePopBackStack
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.qrscanner.QrCodeImage
import java.util.Locale
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds
import kotlin.time.TimeMark

@Composable
fun PairDeviceScreen(viewModel: PairDeviceViewModel = hiltViewModel()) {
	SecureScreen()
	SecureOverlayProtection()

	val navController = LocalNavController.current
	val state by viewModel.state.collectAsStateWithLifecycle()
	val deviceAuth = rememberDeviceAuth { _, cryptoObject -> viewModel.confirm(cryptoObject) }

	PairDeviceContent(
		state = state,
		onNewCode = viewModel::newCode,
		onCodesMatch = { deviceAuth.request(DeviceAuthAction.PAIR_DEVICE) },
		onCodesDontMatch = {
			viewModel.reject()
			navController.safePopBackStack()
		},
		onDone = { navController.safePopBackStack() },
	)
}

@Composable
fun PairDeviceContent(state: PairingState, onNewCode: () -> Unit, onCodesMatch: () -> Unit, onCodesDontMatch: () -> Unit, onDone: () -> Unit, modifier: Modifier = Modifier) {
	Column(
		modifier = modifier
			.fillMaxSize()
			.verticalScroll(rememberScrollState())
			.padding(horizontal = 16.dp.scaledWidth(), vertical = 24.dp)
			.navigationBarsPadding(),
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(24.dp),
	) {
		when (state) {
			is PairingState.ShowingCode -> ShowingCode(state = state, onNewCode = onNewCode)
			is PairingState.ConfirmCode -> ConfirmCode(pin = state.pin, onCodesMatch = onCodesMatch, onCodesDontMatch = onCodesDontMatch)
			PairingState.SharingAccount -> PairingStatus(title = stringResource(R.string.pair_device_sharing), loading = true)
			PairingState.AccountShared -> PairingStatus(
				title = stringResource(R.string.pair_device_done_title),
				body = stringResource(R.string.pair_device_done_body),
				actionLabel = stringResource(R.string.pair_device_done),
				onAction = onDone,
			)
			PairingState.Expired -> PairingStatus(
				title = stringResource(R.string.pairing_expired),
				actionLabel = stringResource(R.string.pair_device_new_code),
				onAction = onNewCode,
			)
			is PairingState.Failed -> PairingStatus(
				title = stringResource(state.failure.messageRes()),
				actionLabel = stringResource(R.string.pairing_try_again),
				onAction = onNewCode,
			)
			else -> PairingStatus(title = stringResource(R.string.pair_device_starting), loading = true)
		}
	}
}

@Composable
private fun ShowingCode(state: PairingState.ShowingCode, onNewCode: () -> Unit, modifier: Modifier = Modifier) {
	val remaining = rememberRemaining(state.expiresAt)
	Column(
		modifier = modifier,
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(24.dp),
	) {
		Text(
			text = stringResource(R.string.pair_device_instructions),
			style = MaterialTheme.typography.bodyLarge,
			color = MaterialTheme.colorScheme.onBackground,
			textAlign = TextAlign.Center,
		)
		QrCodeImage(
			content = state.url,
			contentDescription = stringResource(R.string.pair_device_qr_description),
			modifier = Modifier
				.widthIn(max = 280.dp)
				.fillMaxWidth(),
		)
		if (remaining > Duration.ZERO) {
			Text(
				text = stringResource(R.string.pair_device_expires_in, remaining.formatMinutesSeconds()),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.outline,
			)
		} else {
			PairingStatus(
				title = stringResource(R.string.pairing_expired),
				actionLabel = stringResource(R.string.pair_device_new_code),
				onAction = onNewCode,
			)
		}
	}
}

@Composable
private fun ConfirmCode(pin: String, onCodesMatch: () -> Unit, onCodesDontMatch: () -> Unit, modifier: Modifier = Modifier) {
	Column(
		modifier = modifier,
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(24.dp),
	) {
		PairingStatus(
			title = stringResource(R.string.pair_device_confirm_title),
			body = stringResource(R.string.pair_device_confirm_body),
		)
		PairingPin(pin = pin)
		Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
			PairingButton(text = stringResource(R.string.pair_device_codes_match), onClick = onCodesMatch)
			OutlineStyledButton(
				onClick = onCodesDontMatch,
				content = {
					Text(
						stringResource(R.string.pair_device_codes_dont_match),
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
	}
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
