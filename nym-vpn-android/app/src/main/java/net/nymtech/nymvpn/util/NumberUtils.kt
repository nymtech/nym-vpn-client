package net.nymtech.nymvpn.util

import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

object NumberUtils {
	fun convertSecondsToTimeString(seconds: Long): String {
		val duration: Duration = seconds.seconds
		return duration.toComponents { hour, minute, second, _ ->
			"%02d:%02d:%02d".format(hour, minute, second)
		}
	}
}
