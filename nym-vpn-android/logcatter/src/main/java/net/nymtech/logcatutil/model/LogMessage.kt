package net.nymtech.logcatutil.model

import java.time.Instant
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.ZoneId
import java.time.format.DateTimeFormatter

data class LogMessage(
	val time: String,
	val pid: String,
	val tid: String,
	val level: LogLevel,
	val tag: String,
	val message: String,
) {
	override fun toString(): String = "$time $pid $tid $level $tag message= $message"

	companion object {
		// threadtime:
		// MM-DD HH:MM:SS.mmm  PID  TID  L TAG: message
		private val THREADTIME_REGEX = Regex(
			"""^(\d{2}-\d{2})\s+(\d{2}:\d{2}:\d{2}\.\d{3})\s+(\d+)\s+(\d+)\s+([VDIWEAF])\s+([^:]+):\s?(.*)$""",
		)

		// Used to turn "01-22 12:29:03.228" into a full timestamp string.
		// We assume the current year (good enough for live log viewing).
		private val threadTimeFormatter = DateTimeFormatter.ofPattern("MM-dd HH:mm:ss.SSS")

		fun from(logcatLine: String): LogMessage {
			val line = logcatLine.trim()

			if (line.contains("---------")) return system(line)

			val match = THREADTIME_REGEX.find(line)
				?: return system(line) // fallback instead of crashing

			val (mmdd, hhmmssMs, pid, tid, levelChar, rawTag, msg) = match.destructured

			val timeStr = parseThreadTime("$mmdd $hhmmssMs")

			return LogMessage(
				time = timeStr,
				pid = pid,
				tid = tid,
				level = LogLevel.fromSignifier(levelChar),
				tag = rawTag.trim(),
				message = msg,
			)
		}

		private fun parseThreadTime(value: String): String {
			// value = "01-22 12:29:03.228"
			// Attach current year to keep it sortable and consistent.
			// If parsing fails, return original.
			return runCatching {
				val now = LocalDate.now(ZoneId.systemDefault())
				val parsed = LocalDateTime.parse(value, threadTimeFormatter)
					.withYear(now.year)
				parsed.toString() // e.g. "2026-01-22T12:29:03.228"
			}.getOrElse { value }
		}

		fun system(message: String): LogMessage {
			return LogMessage(
				time = Instant.now().toString(),
				pid = "0",
				tid = "0",
				level = LogLevel.INFO,
				tag = "System",
				message = message,
			)
		}
	}
}
