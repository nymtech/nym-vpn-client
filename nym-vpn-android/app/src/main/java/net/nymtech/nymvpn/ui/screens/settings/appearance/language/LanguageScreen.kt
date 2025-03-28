package net.nymtech.nymvpn.ui.screens.settings.appearance.language

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.runtime.Composable
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
import net.nymtech.nymvpn.util.LocaleUtil
import net.nymtech.nymvpn.util.extensions.capitalize
import net.nymtech.nymvpn.util.extensions.scaledWidth
import java.text.Collator
import java.util.Locale

@Composable
fun LanguageScreen(appUiState: AppUiState, appViewModel: AppViewModel) {
	val collator = Collator.getInstance(Locale.getDefault())

	val locales = LocaleUtil.supportedLocales.map {
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
