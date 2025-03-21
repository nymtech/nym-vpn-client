package net.nymtech.nymvpn.ui.screens.settings.legal

import androidx.annotation.StringRes
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.openWebUrl

@Composable
fun LegalOptions() {
	val context = LocalContext.current
	val navController = LocalNavController.current
	val legalItems = listOf(
		LegalItem(
			titleRes = R.string.terms_of_use,
			action = LegalAction.WebUrl(R.string.terms_link),
		),
		LegalItem(
			titleRes = R.string.privacy_policy,
			action = LegalAction.WebUrl(R.string.privacy_link),
		),
		LegalItem(
			titleRes = R.string.licenses,
			action = LegalAction.Navigate(Route.Licenses),
		),
	)

	SettingsGroup(
		items = legalItems.map { item ->
			SelectionItem(
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						stringResource(item.titleRes),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				onClick = {
					when (item.action) {
						is LegalAction.WebUrl -> context.openWebUrl(context.getString(item.action.urlRes))
						is LegalAction.Navigate -> navController.navigate(item.action.route)
					}
				},
			)
		},
	)
}

data class LegalItem(
	@StringRes val titleRes: Int,
	val action: LegalAction,
)

sealed class LegalAction {
	data class WebUrl(@StringRes val urlRes: Int) : LegalAction()
	data class Navigate(val route: Route) : LegalAction()
}
