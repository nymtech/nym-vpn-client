package net.nymtech.nymvpn.ui.screens.settings.appearance.appicon

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.domain.AppIcon
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class AppIconScreenTest {

	@get:Rule
	val composeRule = createComposeRule()

	@Test
	fun disconnected_tapIcon_showsConfirmDialog_confirmInvokesCallback() {
		var selected: AppIcon? = null
		composeRule.setContent {
			AppIconScreen(currentIcon = AppIcon.DEFAULT, canSwitch = true, onIconSelect = { selected = it })
		}

		composeRule.onNodeWithText(label(R.string.app_icon_calculator)).performClick()
		composeRule.onNodeWithText(label(R.string.app_icon_change_title)).assertExists()

		composeRule.onNodeWithText(label(R.string.app_icon_change_confirm)).performClick()
		composeRule.onNodeWithText(label(R.string.app_icon_change_title)).assertDoesNotExist()
		assert(selected == AppIcon.CALCULATOR) { "expected CALCULATOR to be selected, got $selected" }
	}

	@Test
	fun disconnected_tapIcon_cancelDismissesWithoutInvokingCallback() {
		var selected: AppIcon? = null
		composeRule.setContent {
			AppIconScreen(currentIcon = AppIcon.DEFAULT, canSwitch = true, onIconSelect = { selected = it })
		}

		composeRule.onNodeWithText(label(R.string.app_icon_notes)).performClick()
		composeRule.onNodeWithText(label(R.string.app_icon_change_cancel)).performClick()

		composeRule.onNodeWithText(label(R.string.app_icon_change_title)).assertDoesNotExist()
		assert(selected == null) { "expected no icon selected after cancel, got $selected" }
	}

	@Test
	fun connected_tapIcon_showsErrorText_andDoesNotOpenDialog() {
		var selected: AppIcon? = null
		composeRule.setContent {
			AppIconScreen(currentIcon = AppIcon.DEFAULT, canSwitch = false, onIconSelect = { selected = it })
		}

		composeRule.onNodeWithText(label(R.string.app_icon_disconnect_first)).assertExists()

		composeRule.onNodeWithText(label(R.string.app_icon_calculator)).performClick()
		composeRule.onNodeWithText(label(R.string.app_icon_change_title)).assertDoesNotExist()
		assert(selected == null) { "expected no icon selected while VPN is connected, got $selected" }
	}

	@Test
	fun allIcons_renderPreviewWithoutCrashing() {
		composeRule.setContent {
			AppIconScreen(currentIcon = AppIcon.DEFAULT, canSwitch = true, onIconSelect = {})
		}

		AppIcon.entries.forEach { icon ->
			composeRule.onNodeWithText(label(icon.labelRes)).assertExists()
		}
	}

	private fun label(resId: Int): String = InstrumentationRegistry.getInstrumentation().targetContext.getString(resId)
}
