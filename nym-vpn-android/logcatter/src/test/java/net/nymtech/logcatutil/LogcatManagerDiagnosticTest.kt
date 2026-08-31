package net.nymtech.logcatutil

import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import net.nymtech.logcatutil.model.LogLevel
import org.junit.Assert.assertEquals
import org.junit.Test
import java.nio.file.Files

class LogcatManagerDiagnosticTest {

	private fun manager() = LogcatManager(
		pid = 42,
		logDir = Files.createTempDirectory("logcat-test").toString(),
		maxFileSize = 1024,
		maxFolderSize = 4096,
	)

	@Test
	fun writeDiagnostic_emitsToBufferedAppLogs() = runBlocking {
		val manager = manager()

		manager.writeDiagnostic("app", "PriorExit reason=LOW_MEMORY")

		val replayed = manager.bufferedLogsApp.first()
		assertEquals("PriorExit reason=LOW_MEMORY", replayed.message)
		assertEquals("app", replayed.tag)
		assertEquals(LogLevel.INFO, replayed.level)
		assertEquals("42", replayed.pid)
	}
}
