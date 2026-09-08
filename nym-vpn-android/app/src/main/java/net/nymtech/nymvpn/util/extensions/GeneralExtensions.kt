package net.nymtech.nymvpn.util.extensions

import android.content.Context
import android.provider.Settings
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectMode
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import java.util.Locale
import java.util.MissingResourceException
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

fun Long.convertSecondsToTimeString(): String {
	val duration: Duration = seconds
	return duration.toComponents { hour, minute, second, _ ->
		"%02d:%02d:%02d".format(hour, minute, second)
	}
}

fun String.capitalize(locale: Locale): String = this.replaceFirstChar { if (it.isLowerCase()) it.titlecase(locale) else it.toString() }

fun String.truncateWithEllipsis(length: Int): String = if (this.length <= length) this else "${take(length)}..."

fun NymGateway.toLocale(): Locale? = twoLetterCountryISO?.let { Locale(it, it) }

// Locale.isO3Country throws MissingResourceException for codes without an ISO3 mapping
// (user-assigned codes like "XK"/Kosovo on older ICU data, e.g. Android 9)
fun Locale.iso3CountryOrEmpty(): String = try {
	isO3Country
} catch (_: MissingResourceException) {
	""
}

private const val ALWAYS_ON_VPN_APP = "always_on_vpn_app"

fun isVpnAlwaysOn(context: Context): Boolean = try {
	val alwaysOn = Settings.Secure.getString(context.contentResolver, ALWAYS_ON_VPN_APP)
	alwaysOn == context.packageName
} catch (ex: SecurityException) {
	false
}

fun Tunnel.Mode.toConnectMode(): ConnectMode = when (this) {
	Tunnel.Mode.TWO_HOP_MIXNET -> ConnectMode.FAST
	Tunnel.Mode.FIVE_HOP_MIXNET -> ConnectMode.MIXNET
}
