package net.nymtech.nymvpn.util.extensions

import java.time.Instant
import java.util.Locale
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

fun Instant.durationFromNow(): java.time.Duration {
	return java.time.Duration.between(Instant.now(), this)
}
