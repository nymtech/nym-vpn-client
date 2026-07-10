package net.nymtech.nymvpn.util

import android.content.ComponentName
import android.content.pm.PackageManager
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import net.nymtech.nymvpn.data.domain.AppIcon
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class AppIconUtilTest {

	private val context = ApplicationProvider.getApplicationContext<android.content.Context>()

	@After
	fun resetToDefault() {
		AppIconUtil.switchActiveAlias(context, AppIcon.DEFAULT)
	}

	@Test
	fun getCurrent_defaultsToDefault_onFreshState() {
		AppIconUtil.switchActiveAlias(context, AppIcon.DEFAULT)
		assertEquals(AppIcon.DEFAULT, AppIconUtil.getCurrent(context))
	}

	@Test
	fun switchActiveAlias_enablesTargetAndDisablesOthers() {
		AppIconUtil.switchActiveAlias(context, AppIcon.CALCULATOR)

		assertEquals(AppIcon.CALCULATOR, AppIconUtil.getCurrent(context))
		AppIcon.entries.filterNot { it == AppIcon.CALCULATOR }.forEach { other ->
			val state = context.packageManager.getComponentEnabledSetting(ComponentName(context, other.componentName))
			assertEquals(PackageManager.COMPONENT_ENABLED_STATE_DISABLED, state)
		}
	}

	@Test
	fun getCurrent_reflectsSwitchedAlias_asIfAfterRelaunch() {
		AppIconUtil.switchActiveAlias(context, AppIcon.NOTES)

		val freshRead = AppIconUtil.getCurrent(ApplicationProvider.getApplicationContext())
		assertEquals(AppIcon.NOTES, freshRead)
	}

	@Test
	fun switchActiveAlias_toEveryIcon_isReadBackCorrectly() {
		AppIcon.entries.forEach { icon ->
			AppIconUtil.switchActiveAlias(context, icon)
			assertEquals(icon, AppIconUtil.getCurrent(context))
		}
	}
}
