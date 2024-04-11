package net.nymtech.nymvpn

import io.appium.java_client.AppiumBy
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.android.options.UiAutomator2Options
import net.nymtech.nymvpn.util.Constants
import org.junit.Test
import java.io.File
import java.net.URL

class AppiumSetup {
	private val file = File("build/outputs/apk/fdroid/debug/nymvpn-fdroid-debug-v0.1.1.apk")

	private var options: UiAutomator2Options =
		UiAutomator2Options()
			.setApp(file.absolutePath)
	private var driver: AndroidDriver =
		AndroidDriver(
			URL("http://127.0.0.1:4723"),
			options,
		)

	@Test
	fun testConnection() {
		try {
			val el =
				driver.findElement(
					AppiumBy.androidUIAutomator(
						"UiSelector().resourceId(\"${Constants.CONNECT_TEST_TAG}\")",
					),
				)
			el.click()
			driver.pageSource
		} finally {
			driver.quit()
		}
	}
}
