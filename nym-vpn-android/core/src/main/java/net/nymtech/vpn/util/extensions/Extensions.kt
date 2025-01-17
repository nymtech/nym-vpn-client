package net.nymtech.vpn.util.extensions

import kotlinx.coroutines.delay
import java.util.concurrent.TimeoutException
import java.util.concurrent.atomic.AtomicBoolean

suspend fun AtomicBoolean.waitForTrue(timeout: Long = 5000L) {
	val startTime = System.currentTimeMillis()
	while (System.currentTimeMillis() - startTime < timeout) {
		if (this.get()) {
			return
		}
		delay(10)
	}
	throw TimeoutException()
}
