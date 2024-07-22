package net.nymtech.nymvpn.ui.screens.settings.appearance.language

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import com.zaneschepke.localization_util.LocaleStorage
import com.zaneschepke.localization_util.LocaleUtil
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.buttons.SelectionItemButton
import net.nymtech.nymvpn.ui.common.labels.SelectedLabel
import net.nymtech.nymvpn.util.capitalize
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth
import timber.log.Timber
import java.text.Collator
import java.util.Locale

@Composable
fun LanguageScreen(navController: NavController, localeStorage: LocaleStorage) {
	val context = LocalContext.current

	val collator = Collator.getInstance(Locale.getDefault())

	val currentLocale = remember { mutableStateOf(LocaleUtil.OPTION_PHONE_LANGUAGE) }

	val locales = LocaleUtil.supportedLocales.map {
		val tag = it.replace("_", "-")
		Locale.forLanguageTag(tag)
	}

	val sortedLocales =
		remember(locales) {
			locales.sortedWith(compareBy(collator) { it.getDisplayName(it) })
		}

	LaunchedEffect(Unit) {
		currentLocale.value = localeStorage.getPreferredLocale()
	}

	fun onChangeLocale(locale: String) {
		Timber.d("Setting preferred locale: $locale")
		localeStorage.setPreferredLocale(locale)
		LocaleUtil.applyLocalizedContext(context, locale)
		navController.navigate(NavItem.Main.route)
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
			)
		}
		items(sortedLocales.toList()) { locale ->
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
			)
		}
	}
}
