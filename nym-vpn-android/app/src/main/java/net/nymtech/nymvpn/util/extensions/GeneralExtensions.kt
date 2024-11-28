package net.nymtech.nymvpn.util.extensions

import net.nymtech.vpn.model.Country
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

fun Set<Country>.default(): Country {
	return this.firstOrNull() ?: Country(isDefault = true)
}

fun Long.toMB(): String {
	val mb = this / 1024.0 * 1024.0
	return "%.2f".format(round(mb * 100) / 100)
}
