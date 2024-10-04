package net.nymtech.nymvpn.ui.screens.settings.appearance.language

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.localizationutil.LocaleStorage
import net.nymtech.localizationutil.LocaleUtil
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.SelectionItemButton
import net.nymtech.nymvpn.ui.common.labels.SelectedLabel
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.util.extensions.capitalize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import timber.log.Timber
import java.text.Collator
import java.util.Locale

@Composable
fun LanguageScreen(appViewModel: AppViewModel, localeStorage: LocaleStorage) {
	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle(stringResource(R.string.language)) },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack) {
						appViewModel.navController.popBackStack()
					}
				},
			),
		)
	}

	val context = LocalContext.current

	val collator = Collator.getInstance(Locale.getDefault())

	val currentLocale = remember { mutableStateOf(LocaleUtil.OPTION_PHONE_LANGUAGE) }

	val locales = LocaleUtil.supportedLocales.map {
		val tag = it.replace("_", "-")
		Locale.forLanguageTag(tag)
	}

	val sortedLocales =
		remember(locales) {
			locales.sortedWith(compareBy(collator) { it.getDisplayName(it) }).toList()
		}

	LaunchedEffect(Unit) {
		currentLocale.value = localeStorage.getPreferredLocale()
	}

	fun onChangeLocale(locale: String) {
		Timber.d("Setting preferred locale: $locale")
		localeStorage.setPreferredLocale(locale)
		LocaleUtil.applyLocalizedContext(context, locale)
		appViewModel.navController.navigate(Route.Main(changeLanguage = true))
	}

	LazyColumn(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.Top,
		modifier =
		Modifier
			.fillMaxSize()
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		item {
			SelectionItemButton(
				buttonText = stringResource(R.string.automatic),
				onClick = {
					onChangeLocale(LocaleUtil.OPTION_PHONE_LANGUAGE)
				},
				trailing = {
					if (currentLocale.value == LocaleUtil.OPTION_PHONE_LANGUAGE) {
						SelectedLabel()
					}
				},
				ripple = false,
			)
		}
		items(sortedLocales, key = { it.toLanguageTag() }) { locale ->
			SelectionItemButton(
				buttonText = locale.getDisplayLanguage(locale).capitalize(locale) +
					if (locale.toLanguageTag().contains("-")) " (${locale.getDisplayCountry(locale).capitalize(locale)})" else "",
				onClick = {
					onChangeLocale(locale.toLanguageTag())
				},
				trailing = {
					if (locale.toLanguageTag() == currentLocale.value) {
						SelectedLabel()
					}
				},
				ripple = false,
			)
		}
	}
}
