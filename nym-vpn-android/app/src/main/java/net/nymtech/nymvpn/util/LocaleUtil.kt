package net.nymtech.nymvpn.util

import androidx.appcompat.app.AppCompatDelegate
import androidx.core.os.LocaleListCompat
import net.nymtech.nymvpn.BuildConfig
import java.text.Collator
import java.util.Locale

object LocaleUtil {
	const val OPTION_PHONE_LANGUAGE = "sys_def"

	val supportedLocales: List<Locale> by lazy {
		val collator = Collator.getInstance(Locale.getDefault())
		BuildConfig.LANGUAGES
			.map { it.replace("-r", "-").replace("_", "-") }
			.map { Locale.forLanguageTag(it) }
			.sortedWith(compareBy(collator) { it.getDisplayName(it) })
	}

	fun changeLocale(locale: String) {
		if (locale == OPTION_PHONE_LANGUAGE) return resetToSystemLanguage()
		val appLocale: LocaleListCompat = LocaleListCompat.forLanguageTags(locale)
		AppCompatDelegate.setApplicationLocales(appLocale)
	}

	private fun resetToSystemLanguage() {
		AppCompatDelegate.setApplicationLocales(LocaleListCompat.getEmptyLocaleList())
	}
}
