package net.nymtech.logcathelper

import net.nymtech.logcathelper.model.LogMessage

object LogcatHelper {
	fun logs(callback: (input: LogMessage) -> Unit) {
		clear()
		Runtime.getRuntime().exec("logcat -v epoch")
			.inputStream
			.bufferedReader()
			.useLines { lines ->
				lines.forEach { callback(LogMessage.from(it)) }
			}
	}

	fun clear() {
		Runtime.getRuntime().exec("logcat -c")
	}
}
