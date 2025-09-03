package net.nymtech.nymvpn.util.extensions

import android.content.Context
import android.provider.Settings
import net.nymtech.vpn.model.NymGateway
import java.util.Locale
import kotlin.math.round
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

fun Long.convertSecondsToTimeString(): String {
	val duration: Duration = seconds
	return duration.toComponents { hour, minute, second, _ ->
		"%02d:%02d:%02d".format(hour, minute, second)
	}
}

fun String.capitalize(locale: Locale): String {
	return this.replaceFirstChar { if (it.isLowerCase()) it.titlecase(locale) else it.toString() }
}

fun Long.toMB(): String {
	val mb = this / 1024.0 * 1024.0
	return "%.2f".format(round(mb * 100) / 100)
}

fun String.truncateWithEllipsis(length: Int): String {
	return if (this.length <= length) this else "${take(length)}..."
}

fun NymGateway.toLocale(): Locale? {
	return twoLetterCountryISO?.let { Locale(it, it) }
}

private const val ALWAYS_ON_VPN_APP = "always_on_vpn_app"

fun isVpnAlwaysOn(context: Context): Boolean {
	return try {
		val alwaysOn = Settings.Secure.getString(context.contentResolver, ALWAYS_ON_VPN_APP)
		alwaysOn == context.packageName
	} catch (ex: SecurityException) {
		false
	}
}
