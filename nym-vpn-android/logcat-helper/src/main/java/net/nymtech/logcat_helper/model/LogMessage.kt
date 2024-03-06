package net.nymtech.logcat_helper.model

import java.time.Instant

data class LogMessage(
    val time: Instant,
    val pid: String,
    val tid: String,
    val level : LogLevel,
    val tag: String,
    val message: String
) {

    companion object {
        fun from(logcatLine : String) : LogMessage {
            return if(logcatLine.contains("---------")) LogMessage(Instant.now(), "0","0",LogLevel.VERBOSE,"System", logcatLine)
            else {
                val parts = logcatLine.trim().split(" ")
                val epochParts = parts[0].split(".").map { it.toLong() }
                val message = parts.subList(5, parts.size -1).joinToString(" ")
                LogMessage(Instant.ofEpochSecond(epochParts[0], epochParts[1]), parts[1], parts[2], LogLevel.fromSignifier(parts[3]), parts[4], message)
            }
        }
    }
}
