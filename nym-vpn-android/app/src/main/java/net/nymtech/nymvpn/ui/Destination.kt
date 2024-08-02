package net.nymtech.nymvpn.ui

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.ui.graphics.vector.ImageVector
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.screens.permission.Permission
import net.nymtech.nymvpn.util.StringValue
import kotlin.reflect.full.isSubclassOf

sealed class Destination(
	val route: String,
	val title: StringValue,
	val leading: ImageVector?,
	val trailing: ImageVector? = null,
) {
	data object Main :
		Destination("/main?autoStart={autoStart}", StringValue.StringResource(R.string.app_name), null, settingsIcon) {
		fun createRoute(autoStart: Boolean) = "/main?autoStart=$autoStart"
	}

	data object Analytics : Destination(
		"/analytics",
		StringValue.DynamicString(""),
		backIcon,
	)

	data object Permission : Destination(
		"/permission/{permission}",
		StringValue.StringResource(R.string.permission_required),
		backIcon,
	) {
		fun createRoute(permission: net.nymtech.nymvpn.ui.screens.permission.Permission) = "/permission/${permission.name}"
	}

	data object Settings :
		Destination("/settings", StringValue.StringResource(R.string.settings), backIcon)

	data object Appearance : Destination(
		"/settings/appearance",
		StringValue.StringResource(R.string.appearance),
		backIcon,
	)

	data object Display : Destination(
		"/settings/appearance/display",
		StringValue.StringResource(R.string.display_theme),
		backIcon,
	)
	data object Language : Destination(
		"/settings/appearance/language",
		StringValue.StringResource(R.string.language),
		backIcon,
	)

	data object Logs : Destination(
		"/settings/logs",
		StringValue.StringResource(R.string.logs),
		backIcon,
	)

	data object Feedback : Destination(
		"/settings/feedback",
		StringValue.StringResource(R.string.feedback),
		backIcon,
	)

	data object Support : Destination(
		"/settings/support",
		StringValue.StringResource(R.string.support),
		backIcon,
	)

	data object Legal : Destination(
		"/settings/legal",
		StringValue.StringResource(R.string.legal),
		backIcon,
	)

	data object Licenses : Destination(
		"/settings/legal/licenses",
		StringValue.StringResource(R.string.licenses),
		backIcon,
	)

	data object Credential : Destination(
		"/settings/credential",
		StringValue.DynamicString(""),
		backIcon,
	)

	data object Account : Destination(
		"/settings/account",
		StringValue.StringResource(R.string.credential),
		backIcon,
	)

	data object EntryLocation :
		Destination("/exitLocation", StringValue.StringResource(R.string.entry_location), backIcon, infoIcon)

	data object ExitLocation :
		Destination("/entryLocation", StringValue.StringResource(R.string.exit_location), backIcon, infoIcon)

	companion object {
		val settingsIcon = Icons.Outlined.Settings
		val backIcon = Icons.AutoMirrored.Filled.ArrowBack
		val infoIcon = Icons.Outlined.Info

		@JvmStatic private val map = Destination::class.nestedClasses
			.filter { klass -> klass.isSubclassOf(Destination::class) }
			.map { klass -> klass.objectInstance }
			.filterIsInstance<Destination>()
			.associateBy { value -> value.route }

		@JvmStatic fun valueOf(value: String) = requireNotNull(map[value]) {
			"No enum constant ${Destination::class.java.name}.$value"
		}

		fun from(route: String?): Destination {
			return route?.let {
				try {
					valueOf(route)
				} catch (_: IllegalArgumentException) {
					Main
				}
			} ?: Main
		}
	}
}
