package net.nymtech.nymvpn.data.datastore

import android.content.Context
import android.content.SharedPreferences
import net.nymtech.nymvpn.util.LocaleUtil

class LocaleStorage(context : Context) {
	private var preferences: SharedPreferences = context.getSharedPreferences("sp", Context.MODE_PRIVATE)

	fun getPreferredLocale(): String {
		return preferences.getString("preferred_locale", LocaleUtil.OPTION_PHONE_LANGUAGE)!!
	}

	fun setPreferredLocale(localeCode: String) {
		preferences.edit().putString("preferred_locale", localeCode).apply()
	}
}
