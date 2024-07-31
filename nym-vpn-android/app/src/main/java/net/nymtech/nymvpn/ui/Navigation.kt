package net.nymtech.nymvpn.ui

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.ui.graphics.vector.ImageVector
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.StringValue

enum class Screen {
	MAIN,
	SETTINGS,
	Location,
	DISPLAY,
	LOGS,
	FEEDBACK,
	LEGAL,
	SUPPORT,
	CREDENTIAL,
	ACCOUNT,
	LICENSES,
	ANALYTICS,
	PERMISSION,
	LANGUAGE,
	APPEARANCE,
}

enum class GatewayLocation {
	Entry,
	Exit,
	;

	fun title(): StringValue {
		return when (this) {
			Entry -> StringValue.StringResource(R.string.entry_location)
			Exit -> StringValue.StringResource(R.string.exit_location)
		}
	}
}

sealed class NavItem(
	val route: String,
	val title: StringValue,
	val leading: ImageVector?,
	val trailing: ImageVector? = null,
) {
	data object Main :
		NavItem(Screen.MAIN.name, StringValue.StringResource(R.string.app_name), null, settingsIcon)

	data object Analytics : NavItem(
		Screen.ANALYTICS.name,
		StringValue.DynamicString(""),
		backIcon,
	)

	data object Permission : NavItem(
		Screen.PERMISSION.name,
		StringValue.StringResource(R.string.permission_required),
		backIcon,
	) {
		enum class Path {
			VPN,
			NOTIFICATION,
		}
	}

	data object Settings :
		NavItem(Screen.SETTINGS.name, StringValue.StringResource(R.string.settings), backIcon) {

		data object Logs : NavItem(
			"${Screen.SETTINGS.name}/${Screen.LOGS.name}",
			StringValue.StringResource(R.string.logs),
			backIcon,
		)

		data object Feedback : NavItem(
			"${Screen.SETTINGS.name}/${Screen.FEEDBACK.name}",
			StringValue.StringResource(R.string.feedback),
			backIcon,
		)

		data object Support : NavItem(
			"${Screen.SETTINGS.name}/${Screen.SUPPORT.name}",
			StringValue.StringResource(R.string.support),
			backIcon,
		)

		data object Legal : NavItem(
			"${Screen.SETTINGS.name}/${Screen.LEGAL.name}",
			StringValue.StringResource(R.string.legal),
			backIcon,
		) {
			data object Licenses : NavItem(
				"${Screen.SETTINGS.name}/${Screen.LEGAL.name}/${Screen.LICENSES.name}",
				StringValue.StringResource(R.string.licenses),
				backIcon,
			)
		}

		data object Credential : NavItem(
			"${Screen.SETTINGS.name}/${Screen.CREDENTIAL.name}",
			StringValue.DynamicString(""),
			backIcon,
		)

		data object Account : NavItem(
			"${Screen.SETTINGS.name}/${Screen.ACCOUNT.name}",
			StringValue.StringResource(R.string.credential),
			backIcon,
		)

		data object Appearance : NavItem(
			"${Screen.SETTINGS.name}/${Screen.APPEARANCE.name}",
			StringValue.StringResource(R.string.appearance),
			backIcon,
		) {
			data object Display : NavItem(
				"${Screen.SETTINGS.name}/${Screen.APPEARANCE.name}/${Screen.DISPLAY.name}",
				StringValue.StringResource(R.string.display_theme),
				backIcon,
			)
			data object Language : NavItem(
				"${Screen.SETTINGS.name}/${Screen.APPEARANCE.name}/${Screen.LANGUAGE.name}",
				StringValue.StringResource(R.string.language),
				backIcon,
			)
		}
	}

	sealed class Location {
		data object Entry :
			NavItem("${Screen.Location.name}/${GatewayLocation.Entry.name}", GatewayLocation.Entry.title(), backIcon, infoIcon)

		data object Exit :
			NavItem("${Screen.Location.name}/${GatewayLocation.Exit.name}", GatewayLocation.Exit.title(), backIcon, infoIcon)
	}

	companion object {
		val settingsIcon = Icons.Outlined.Settings
		val backIcon = Icons.AutoMirrored.Filled.ArrowBack
		val infoIcon = Icons.Outlined.Info

		fun from(route: String?): NavItem {
			return with(route) {
				when {
					equals(Main.route) -> Main
					equals(Analytics.route) -> Analytics
					this?.contains(Permission.route) == true -> Permission
					equals(Settings.route) -> Settings
					equals(Location.Entry.route) -> Location.Entry
					equals(Location.Exit.route) -> Location.Exit
					equals(Settings.Appearance.Display.route) -> Settings.Appearance.Display
					equals(Settings.Logs.route) -> Settings.Logs
					equals(Settings.Support.route) -> Settings.Support
					equals(Settings.Feedback.route) -> Settings.Feedback
					equals(Settings.Legal.route) -> Settings.Legal
					equals(Settings.Credential.route) -> Settings.Credential
					equals(Settings.Account.route) -> Settings.Account
					equals(Settings.Appearance.route) -> Settings.Appearance
					equals(Settings.Appearance.Display.route) -> Settings.Appearance.Display
					equals(Settings.Appearance.Language.route) -> Settings.Appearance.Language
					equals(Settings.Legal.Licenses.route) -> Settings.Legal.Licenses
					else -> Main
				}
			}
		}
	}
}
