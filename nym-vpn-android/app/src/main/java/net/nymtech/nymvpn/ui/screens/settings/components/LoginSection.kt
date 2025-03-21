package net.nymtech.nymvpn.ui.screens.settings.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun LoginSection(appUiState: AppUiState, onLoginClick: () -> Unit) {
	if (!appUiState.managerState.isMnemonicStored) {
		MainStyledButton(
			onClick = onLoginClick,
			content = { Text(stringResource(R.string.log_in), style = CustomTypography.labelHuge) },
			color = MaterialTheme.colorScheme.primary,
			modifier = Modifier.fillMaxWidth().height(56.dp.scaledHeight()),
		)
	}
}

@Composable
fun LogoutSection(appUiState: AppUiState, loggingOut: Boolean, onLogoutClick: () -> Unit) {
	if (appUiState.managerState.isMnemonicStored) {
		SettingsGroup(
			items = listOf(
				SelectionItem(
					title = {
						Text(
							if (loggingOut) stringResource(R.string.logging_out) else stringResource(R.string.log_out),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					onClick = onLogoutClick,
				),
			),
		)
	}
}
