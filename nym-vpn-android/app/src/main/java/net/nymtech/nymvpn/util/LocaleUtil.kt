package net.nymtech.nymvpn.util

import android.content.res.Resources
import android.os.Build
import androidx.appcompat.app.AppCompatDelegate
import androidx.core.content.ContextCompat.getSystemService
import androidx.core.os.ConfigurationCompat
import androidx.core.os.LocaleListCompat
import net.nymtech.nymvpn.BuildConfig
import timber.log.Timber

object LocaleUtil {
	private const val DEFAULT_LANG = "en"
	val supportedLocales: Array<String> = BuildConfig.LANGUAGES
	const val OPTION_PHONE_LANGUAGE = "sys_def"

	/**
	 * returns the locale to use depending on the preference value
	 * when preference value = "sys_def" returns the locale of current system
	 * else it returns the locale code e.g. "en", "bn" etc.
	 */
	fun getLocaleFromPrefCode(prefCode: String): String {
		val localeCode = if (prefCode != OPTION_PHONE_LANGUAGE) {
			prefCode
		} else {
			val systemLang = ConfigurationCompat.getLocales(Resources.getSystem().configuration).get(0)?.language ?: DEFAULT_LANG
			if (systemLang in supportedLocales) {
				systemLang
			} else {
				DEFAULT_LANG
			}
		}
		return localeCode
	}

	fun changeLocale(locale: String) {
		if(locale == OPTION_PHONE_LANGUAGE) return resetToSystemLanguage()
		val appLocale: LocaleListCompat = LocaleListCompat.forLanguageTags(locale)
		AppCompatDelegate.setApplicationLocales(appLocale)
	}

	private fun resetToSystemLanguage() {
		AppCompatDelegate.setApplicationLocales(LocaleListCompat.getEmptyLocaleList())
	}
}
