package net.nymtech.logcatutil.model

enum class LogType(val prefix: String) {
	APP("app"),
	TUNNEL("tunnel"),
	CORE("core"),
	LOGCAT("logcat"),
}
