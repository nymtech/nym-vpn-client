package net.nymtech.nymvpn.ui.screens.account.pairing

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.pairing.model.PairingState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.functions.SecureScreen
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.security.SecureOverlayProtection
import net.nymtech.nymvpn.ui.screens.account.pairing.components.PairingPin
import net.nymtech.nymvpn.ui.screens.account.pairing.components.PairingStatus
import net.nymtech.nymvpn.util.extensions.messageRes
import net.nymtech.nymvpn.util.extensions.navigateAndForgetToMain
import net.nymtech.nymvpn.util.extensions.safePopBackStack
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun PairingSignInScreen(viewModel: PairingSignInViewModel = hiltViewModel()) {
	SecureScreen()
	SecureOverlayProtection()

	val navController = LocalNavController.current
	val state by viewModel.state.collectAsStateWithLifecycle()

	LaunchedEffect(state) {
		// The core has already stored the account.
		if (state == PairingState.SignedIn) {
			navController.navigateAndForgetToMain(Route.Main(autoStart = false, loginProcessing = true))
		}
	}

	PairingSignInContent(state = state, onBack = { navController.safePopBackStack() })
}

@Composable
fun PairingSignInContent(state: PairingState, onBack: () -> Unit, modifier: Modifier = Modifier) {
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
			is PairingState.WaitingForApproval -> {
				PairingStatus(
					title = stringResource(R.string.pairing_sign_in_title),
					body = stringResource(R.string.pairing_sign_in_body),
				)
				PairingPin(pin = state.pin)
				CircularProgressIndicator(color = MaterialTheme.colorScheme.primary)
				TextButton(onClick = onBack) {
					Text(stringResource(R.string.pairing_cancel), style = MaterialTheme.typography.titleMedium)
				}
			}
			PairingState.SignedIn -> PairingStatus(title = stringResource(R.string.pairing_signing_in), loading = true)
			PairingState.Expired -> PairingStatus(
				title = stringResource(R.string.pairing_expired),
				actionLabel = stringResource(R.string.pairing_back),
				onAction = onBack,
			)
			is PairingState.Failed -> PairingStatus(
				title = stringResource(state.failure.messageRes()),
				actionLabel = stringResource(R.string.pairing_back),
				onAction = onBack,
			)
			// Idle after process death or a cancel.
			else -> PairingStatus(
				title = stringResource(R.string.pairing_ended),
				actionLabel = stringResource(R.string.pairing_back),
				onAction = onBack,
			)
		}
	}
}
