package net.nymtech.nymvpn.ui

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.outlined.DeleteForever
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.ui.graphics.vector.ImageVector
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.StringValue

enum class Screen {
    MAIN,
    SETTINGS,
    HOP,
    DISPLAY,
    LOGS,
    FEEDBACK,
    LEGAL,
    SUPPORT,
    LOGIN,
    ACCOUNT,
    LICENSES,
}

enum class HopType {
    FIRST,
    LAST;

    fun hopTitle(): StringValue {
        return when (this) {
            FIRST -> StringValue.StringResource(R.string.first_hop_selection)
            LAST -> StringValue.StringResource(R.string.last_hop_selection)
        }
    }
}

sealed class NavItem(
    val route: String,
    val title: StringValue,
    val leading: ImageVector?,
    val trailing: ImageVector? = null
) {
    data object Main :
        NavItem(Screen.MAIN.name, StringValue.StringResource(R.string.app_name), null, settingsIcon)

    data object Settings :
        NavItem(Screen.SETTINGS.name, StringValue.StringResource(R.string.settings), backIcon) {
        data object Display : NavItem(
            "${Screen.SETTINGS.name}/${Screen.DISPLAY.name}",
            StringValue.StringResource(R.string.display_theme),
            backIcon
        )

        data object Logs : NavItem(
            "${Screen.SETTINGS.name}/${Screen.LOGS.name}",
            StringValue.StringResource(R.string.logs),
            backIcon,
            trailing = clearLogsIcon
        )

        data object Feedback : NavItem(
            "${Screen.SETTINGS.name}/${Screen.FEEDBACK.name}",
            StringValue.StringResource(R.string.feedback),
            backIcon
        )

        data object Support : NavItem(
            "${Screen.SETTINGS.name}/${Screen.SUPPORT.name}",
            StringValue.StringResource(R.string.support),
            backIcon
        )

        data object Legal : NavItem(
            "${Screen.SETTINGS.name}/${Screen.LEGAL.name}",
            StringValue.StringResource(R.string.legal),
            backIcon
        ) {
            data object Licenses : NavItem(
                "${Screen.SETTINGS.name}/${Screen.LEGAL.name}/${Screen.LICENSES.name}",
                StringValue.StringResource(R.string.licenses),
                backIcon
            )
        }

        data object Login : NavItem(
            "${Screen.SETTINGS.name}/${Screen.LOGIN.name}",
            StringValue.DynamicString(""),
            backIcon
        )

        data object Account : NavItem(
            "${Screen.SETTINGS.name}/${Screen.ACCOUNT.name}",
            StringValue.StringResource(R.string.credential),
            backIcon
        )
    }

    sealed class Hop {
        data object Entry :
            NavItem("${Screen.HOP.name}/${HopType.FIRST.name}", HopType.FIRST.hopTitle(), backIcon)

        data object Exit :
            NavItem("${Screen.HOP.name}/${HopType.LAST.name}", HopType.LAST.hopTitle(), backIcon)
    }

    companion object {
        val settingsIcon = Icons.Outlined.Settings
        val backIcon = Icons.AutoMirrored.Filled.ArrowBack
        val clearLogsIcon = Icons.Outlined.DeleteForever
        fun from(route: String?): NavItem {
            return when (route) {
                Main.route -> Main
                Settings.route -> Settings
                Hop.Entry.route -> Hop.Entry
                Hop.Exit.route -> Hop.Exit
                Settings.Display.route -> Settings.Display
                Settings.Logs.route -> Settings.Logs
                Settings.Support.route -> Settings.Support
                Settings.Feedback.route -> Settings.Feedback
                Settings.Legal.route -> Settings.Legal
                Settings.Login.route -> Settings.Login
                Settings.Account.route -> Settings.Account
                Settings.Legal.Licenses.route -> Settings.Legal.Licenses
                else -> Main
            }
        }
    }
}
