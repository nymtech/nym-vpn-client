package net.nymtech.nymvpn.util

import androidx.appcompat.app.AppCompatDelegate
import androidx.core.os.LocaleListCompat
import net.nymtech.nymvpn.BuildConfig

object LocaleUtil {
	val supportedLocales: Array<String> = BuildConfig.LANGUAGES
	const val OPTION_PHONE_LANGUAGE = "sys_def"

	fun changeLocale(locale: String) {
		if (locale == OPTION_PHONE_LANGUAGE) return resetToSystemLanguage()
		val tag = locale.replace("r", "").replace("_", "-")
		val appLocale: LocaleListCompat = LocaleListCompat.forLanguageTags(tag)
		AppCompatDelegate.setApplicationLocales(appLocale)
	}

	private fun resetToSystemLanguage() {
		AppCompatDelegate.setApplicationLocales(LocaleListCompat.getEmptyLocaleList())
	}
}
