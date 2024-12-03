package net.nymtech.nymvpn.ui.screens.settings.appearance.language

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.buttons.SelectionItemButton
import net.nymtech.nymvpn.ui.common.labels.SelectedLabel
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.util.LocaleUtil
import net.nymtech.nymvpn.util.extensions.capitalize
import net.nymtech.nymvpn.util.extensions.scaledWidth
import java.text.Collator
import java.util.Locale

@Composable
fun LanguageScreen(appUiState: AppUiState, appViewModel: AppViewModel) {
	val navController = LocalNavController.current

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle(stringResource(R.string.language)) },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack) {
						navController.popBackStack()
					}
				},
			),
		)
	}

	val collator = Collator.getInstance(Locale.getDefault())

	// TODO re-enable fa
	val locales = LocaleUtil.supportedLocales.filter { it != "fa" }.map {
		val tag = it.replace("_", "-")
		Locale.forLanguageTag(tag)
	}

	val sortedLocales =
		remember(locales) {
			locales.sortedWith(compareBy(collator) { it.getDisplayName(it) }).toList()
		}

	LazyColumn(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.Top,
		modifier =
		Modifier
			.fillMaxSize()
			.padding(horizontal = 24.dp.scaledWidth()).windowInsetsPadding(WindowInsets.navigationBars),
	) {
		item {
			SelectionItemButton(
				buttonText = stringResource(R.string.automatic),
				onClick = {
					appViewModel.onLocaleChange(LocaleUtil.OPTION_PHONE_LANGUAGE)
				},
				trailing = {
					if (appUiState.settings.locale == LocaleUtil.OPTION_PHONE_LANGUAGE) {
						SelectedLabel()
					}
				},
				ripple = false,
			)
		}
		items(sortedLocales, key = { it }) { locale ->
			SelectionItemButton(
				buttonText = locale.getDisplayLanguage(locale).capitalize(locale) +
					if (locale.toLanguageTag().contains("-")) " (${locale.getDisplayCountry(locale).capitalize(locale)})" else "",
				onClick = {
					appViewModel.onLocaleChange(locale.toLanguageTag())
				},
				trailing = {
					if (locale.toLanguageTag() == appUiState.settings.locale) {
						SelectedLabel()
					}
				},
				ripple = false,
			)
		}
	}
}
