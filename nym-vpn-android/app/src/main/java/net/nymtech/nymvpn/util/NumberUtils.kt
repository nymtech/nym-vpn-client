package net.nymtech.nymvpn.util

import java.util.concurrent.TimeUnit

object NumberUtils {
    fun convertSecondsToTimeString(seconds : Long) : String {
        return String.format("%02d:%02d:%02d",
            TimeUnit.SECONDS.toHours(seconds),
            TimeUnit.SECONDS.toMinutes(seconds) -
                    TimeUnit.HOURS.toMinutes(TimeUnit.SECONDS.toHours(seconds)),
            TimeUnit.SECONDS.toSeconds(seconds) -
                    TimeUnit.MINUTES.toSeconds(TimeUnit.SECONDS.toMinutes(seconds)))
    }
}