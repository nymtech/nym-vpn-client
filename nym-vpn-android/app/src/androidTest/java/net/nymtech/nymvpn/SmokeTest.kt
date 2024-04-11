package net.nymtech.nymvpn

import android.content.Intent
import android.os.SystemClock
import androidx.test.core.app.ApplicationProvider
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.By
import androidx.test.uiautomator.BySelector
import androidx.test.uiautomator.SearchCondition
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObject2
import androidx.test.uiautomator.UiSelector
import androidx.test.uiautomator.Until
import junit.framework.TestCase.assertNotNull
import net.nymtech.nymvpn.util.Constants
import org.junit.Test
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

class SmokeTest {
	@Test
	fun openApp() {
		val device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())

		startAppAndWait(device)
		device.testVpnConnect()
	}

	private fun startAppAndWait(device: UiDevice) {
		device.pressHome()

		// Wait for launcher
		val launcherPackage = device.launcherPackageName
		assertNotNull(launcherPackage)
		device.wait(Until.hasObject(By.pkg(launcherPackage).depth(0)), 5_000)

		// Launch the app
		val context = ApplicationProvider.getApplicationContext<NymVpn>()
		val packageName = context.packageName
		val intent =
			context.packageManager.getLaunchIntentForPackage(packageName)!!.apply {
				// Clear out any previous instances
				addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK)
			}
		context.startActivity(intent)

		// Wait for the app to appear
		device.wait(Until.hasObject(By.pkg(packageName).depth(0)), 5_000)
	}

	private fun UiDevice.testVpnConnect() {
		connect()
		login()
		connect()
		acceptNotificationPermission()
		connect()
		allowVpnPermission()
		connect()
		disconnect()
		connect()
		disconnect()
	}

	private fun UiDevice.allowVpnPermission() {
		findObject(UiSelector().text("OK")).click()
		waitForIdle()
	}

	private fun UiDevice.acceptNotificationPermission() {
		findObject(UiSelector().text("Allow")).click()
		waitForIdle()
	}

	private fun UiDevice.login() {
		// Open a show from one of the carousels
		runAction(By.res(Constants.LOGIN_TEST_TAG)) { click() }
		waitForIdle()
	}

	private fun UiDevice.connect() {
		// Open a show from one of the carousels'
		waitForIdle()
		kotlin.runCatching {
			SystemClock.sleep(5000)
			findObject(By.res(Constants.CONNECT_TEST_TAG)).click()
		}
		waitForIdle()
	}

	private fun UiDevice.disconnect() {
		waitForIdle()
		kotlin.runCatching {
			SystemClock.sleep(5000)
			findObject(By.res(Constants.DISCONNECT_TEST_TAG)).click()
		}
		waitForIdle()
	}

	private fun UiDevice.runAction(selector: BySelector, maxRetries: Int = 6, action: UiObject2.() -> Unit) {
		waitForObject(selector)

		retry(maxRetries = maxRetries, delay = 5.seconds) {
			// Wait for idle, to avoid recompositions causing StaleObjectExceptions
			waitForIdle()
			requireNotNull(findObject(selector)).action()
		}
	}

	private fun retry(maxRetries: Int, delay: Duration, block: () -> Unit) {
		repeat(maxRetries) { run ->
			val result = runCatching { block() }
			if (result.isSuccess) {
				return
			}
			if (run == maxRetries - 1) {
				result.getOrThrow()
			} else {
				SystemClock.sleep(delay.inWholeMilliseconds)
			}
		}
	}

	private fun UiDevice.waitForObject(selector: BySelector, timeout: Duration = 5.seconds): UiObject2 {
		if (wait(Until.hasObject(selector), timeout)) {
			return findObject(selector)
		}
		error("Object with selector [$selector] not found")
	}

	private fun <R> UiDevice.wait(condition: SearchCondition<R>, timeout: Duration): R {
		return wait(condition, timeout.inWholeMilliseconds)
	}
}
