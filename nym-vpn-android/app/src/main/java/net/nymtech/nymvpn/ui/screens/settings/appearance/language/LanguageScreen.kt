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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.datastore.LocaleStorage
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.buttons.SelectionItemButton
import net.nymtech.nymvpn.ui.common.labels.SelectedLabel
import net.nymtech.nymvpn.util.LocaleUtil
import net.nymtech.nymvpn.util.navigateNoBack
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth
import java.text.Collator
import java.util.Locale

@Composable
fun LanguageScreen(
	navController: NavController,
	localeStorage: LocaleStorage,
	recreate: () -> Unit
) {

	val collator = Collator.getInstance(Locale.getDefault())

	val currentLocale = remember { mutableStateOf(LocaleUtil.OPTION_PHONE_LANGUAGE) }

	val locales = BuildConfig.LANGUAGES.map {
		val tag = it.replace("_", "-")
		Locale.forLanguageTag(tag)
	}

	val sortedLocales =
		remember(locales) {
			locales.sortedWith(compareBy(collator) { it.displayName })
		}

	LaunchedEffect(Unit) {
		currentLocale.value = localeStorage.getPreferredLocale()
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
				{},
				buttonText = stringResource(R.string.automatic),
				onClick = {
					localeStorage.setPreferredLocale(LocaleUtil.OPTION_PHONE_LANGUAGE)
					LocaleUtil.applyLocalizedContext(NymVpn.instance, LocaleUtil.OPTION_PHONE_LANGUAGE)
					recreate()
				},
				trailing = {
					if (currentLocale.value == LocaleUtil.OPTION_PHONE_LANGUAGE) {
						SelectedLabel()
					}
				},
			)
		}
		items(sortedLocales.toList()) {
			SelectionItemButton(
				{},
				buttonText = it.displayName,
				onClick = {
					val lang = it.toLanguageTag()
					localeStorage.setPreferredLocale(lang)
					LocaleUtil.applyLocalizedContext(NymVpn.instance,lang)
					recreate()
				},
				trailing = {
					if (it.toLanguageTag() == currentLocale.value) {
						SelectedLabel()
					}
				},
			)
		}
	}
}
