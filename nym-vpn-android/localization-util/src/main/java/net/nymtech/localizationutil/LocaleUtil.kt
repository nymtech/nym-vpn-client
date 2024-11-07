package net.nymtech.localizationutil

import android.content.Context
import android.content.ContextWrapper
import android.content.res.Resources
import androidx.core.os.ConfigurationCompat
import java.util.Locale

object LocaleUtil {
	private const val DEFAULT_LANG = "en"
	val supportedLocales: Array<String> = BuildConfig.LANGUAGES
	const val OPTION_PHONE_LANGUAGE = "sys_def"

	fun getLocaleFromPrefCode(prefCode: String): Locale {
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
		return Locale.forLanguageTag(localeCode)
	}

	// Update locale on the fly (will not work for things with built-in translations like locale names which require activity restart)
	fun updateLocale(context: Context, lang: String): ContextWrapper {
		val locale = getLocaleFromPrefCode(lang)
		Locale.setDefault(locale)
		val resources = context.resources
		val configuration = resources.configuration
		configuration.setLocale(locale)
		configuration.setLayoutDirection(locale)
		return ContextWrapper(context.createConfigurationContext(configuration))
	}
}
