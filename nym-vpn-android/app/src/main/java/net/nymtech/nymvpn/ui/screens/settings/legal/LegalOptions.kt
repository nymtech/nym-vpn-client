package net.nymtech.nymvpn.ui.screens.settings.legal

import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsArrowIcon
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsTitle
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
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(stringResource(item.titleRes))
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

data class LegalItem(@StringRes val titleRes: Int, val action: LegalAction)

sealed class LegalAction {
	data class WebUrl(@StringRes val urlRes: Int) : LegalAction()
	data class Navigate(val route: Route) : LegalAction()
}
