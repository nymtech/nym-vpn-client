package net.nymtech.nymvpn.ui.screens.account.redeem

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.util.FreepassError
import net.nymtech.nymvpn.util.extensions.navigateAndForget

@Composable
fun RedeemVoucherScreen(viewModel: RedeemVoucherViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val state by viewModel.state.collectAsStateWithLifecycle()

	when (val current = state) {
		RedeemVoucherViewModel.State.Applying -> {
			Column(
				modifier = Modifier
					.fillMaxSize()
					.background(MaterialTheme.colorScheme.background)
					.padding(32.dp),
				horizontalAlignment = Alignment.CenterHorizontally,
				verticalArrangement = Arrangement.Center,
			) {
				CircularProgressIndicator()
				Text(
					text = stringResource(R.string.freepass_redeem_applying),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground,
					modifier = Modifier.padding(top = 24.dp),
				)
			}
		}

		RedeemVoucherViewModel.State.Success -> {
			FreepassSuccessContent(onContinue = { navController.navigateAndForget(Route.Main()) })
		}

		is RedeemVoucherViewModel.State.Error -> {
			val (titleRes, messageRes) = when (current.kind) {
				FreepassError.INVALID ->
					R.string.freepass_error_invalid_title to R.string.freepass_error_invalid_message
				FreepassError.ALREADY_REDEEMED ->
					R.string.freepass_error_redeemed_title to R.string.freepass_error_redeemed_message
				FreepassError.GENERIC ->
					R.string.freepass_error_generic_title to R.string.freepass_error_generic_message
			}
			AlertDialog(
				onDismissRequest = { },
				title = { Text(stringResource(titleRes)) },
				text = { Text(stringResource(messageRes)) },
				confirmButton = {
					TextButton(onClick = {
						navController.navigate(Route.FreepassScanner(existingAccount = true)) {
							popUpTo(Route.RedeemVoucher(code = "")) { inclusive = true }
						}
					}) { Text(stringResource(R.string.freepass_error_try_another)) }
				},
				dismissButton = {
					TextButton(onClick = { navController.popBackStack() }) {
						Text(stringResource(R.string.freepass_error_back))
					}
				},
			)
		}
	}
}
