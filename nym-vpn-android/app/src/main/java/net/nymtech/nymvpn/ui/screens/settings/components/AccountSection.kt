package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.Context
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material.icons.automirrored.outlined.Launch
import androidx.compose.material.icons.outlined.Key
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledWidth
import timber.log.Timber

@Composable
fun AccountSection(appUiState: AppUiState, context: Context) {
	val navController = LocalNavController.current
	if (appUiState.managerState.isMnemonicStored) {
		SettingsGroup(
			items = listOf(
				SelectionItem(
					leading = {
						Icon(
							Icons.Outlined.Person,
							stringResource(R.string.account),
							modifier = Modifier.size(iconSize.scaledWidth()),
						)
					},
					trailing = {
						Icon(
							Icons.AutoMirrored.Outlined.Launch,
							stringResource(R.string.go),
							modifier = Modifier.size(iconSize),
						)
					},
					title = {
						Text(
							stringResource(R.string.account),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					onClick = {
						appUiState.managerState.accountLinks?.account?.let {
							Timber.d("Account url: $it")
							context.openWebUrl(it)
						}
					},
				),

				SelectionItem(
					leading = {
						Icon(
							Icons.Outlined.Key,
							stringResource(R.string.settings_passphrase_title),
							modifier = Modifier.size(iconSize.scaledWidth()),
						)
					},
					trailing = {
						Icon(
							Icons.AutoMirrored.Outlined.ArrowRight,
							stringResource(R.string.go),
							modifier = Modifier.size(iconSize),
						)
					},
					title = {
						Text(
							stringResource(R.string.settings_passphrase_title),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					onClick = {
						navController.navigate(Route.Passphrase)
					},
				),
			),
		)
	}
}
