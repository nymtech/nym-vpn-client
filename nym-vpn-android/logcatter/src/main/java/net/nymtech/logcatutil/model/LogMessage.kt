package net.nymtech.logcatutil.model

import java.time.Instant
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.ZoneId
import java.time.format.DateTimeFormatter

data class LogMessage(val time: String, val epochMillis: Long, val pid: String, val tid: String, val level: LogLevel, val tag: String, val message: String) {
	override fun toString(): String = "$time $pid $tid $level $tag message= $message"

	companion object {
		private val THREADTIME_REGEX = Regex(
			"""^(\d{2}-\d{2})\s+(\d{2}:\d{2}:\d{2}\.\d{3})\s+(\d+)\s+(\d+)\s+([VDIWEAF])\s+([^:]+):\s?(.*)$""",
		)

		private val threadTimeFormatter = DateTimeFormatter.ofPattern("MM-dd HH:mm:ss.SSS")

		fun tryFromThreadtime(logcatLine: String): LogMessage? {
			val line = logcatLine.trimEnd()
			if (line.contains("---------")) return system(line)

			val match = THREADTIME_REGEX.find(line) ?: return null
			val (mmdd, hhmmssMs, pid, tid, levelChar, rawTag, msg) = match.destructured

			val (timeStr, epoch) = parseThreadTimeParts("$mmdd $hhmmssMs")

			return LogMessage(
				time = timeStr,
				epochMillis = epoch,
				pid = pid,
				tid = tid,
				level = LogLevel.fromSignifier(levelChar),
				tag = rawTag.trim(),
				message = msg,
			)
		}

		fun from(logcatLine: String): LogMessage {
			val line = logcatLine.trimEnd()

			if (line.contains("---------")) return system(line)

			val match = THREADTIME_REGEX.find(line)
				?: return system(line)

			val (mmdd, hhmmssMs, pid, tid, levelChar, rawTag, msg) = match.destructured
			val (timeStr, epoch) = parseThreadTimeParts("$mmdd $hhmmssMs")

			return LogMessage(
				time = timeStr,
				epochMillis = epoch,
				pid = pid,
				tid = tid,
				level = LogLevel.fromSignifier(levelChar),
				tag = rawTag.trim(),
				message = msg,
			)
		}

		private fun parseThreadTimeParts(value: String): Pair<String, Long> = runCatching {
			val zone = ZoneId.systemDefault()
			val now = LocalDate.now(zone)
			val parsedLocal = LocalDateTime.parse(value, threadTimeFormatter)
				.withYear(now.year)

			val epoch = parsedLocal
				.atZone(zone)
				.toInstant()
				.toEpochMilli()

			parsedLocal.toString() to epoch
		}.getOrElse {
			val now = Instant.now()
			value to now.toEpochMilli()
		}

		fun system(message: String): LogMessage {
			val now = Instant.now()
			return LogMessage(
				time = now.toString(),
				epochMillis = now.toEpochMilli(),
				pid = "0",
				tid = "0",
				level = LogLevel.INFO,
				tag = "System",
				message = message,
			)
		}
	}
}
