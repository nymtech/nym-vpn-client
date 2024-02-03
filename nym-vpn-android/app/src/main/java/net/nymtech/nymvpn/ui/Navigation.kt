package net.nymtech.nymvpn.ui

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
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
    REVIEW,
    SUPPORT
}

enum class HopType {
    FIRST,
    LAST;
    fun hopTitle() : StringValue {
        return when(this) {
            FIRST -> StringValue.StringResource(R.string.first_hop_selection)
            LAST -> StringValue.StringResource(R.string.last_hop_selection)
        }
    }
}

sealed class NavItem(val route: String, val title : StringValue, val leading : ImageVector?, val trailing : ImageVector? = null) {
    data object Main : NavItem(Screen.MAIN.name, StringValue.StringResource(R.string.app_name), null, settingsIcon)
    data object Settings : NavItem(Screen.SETTINGS.name, StringValue.StringResource(R.string.settings), backIcon) {
        data object Display : NavItem("${Screen.SETTINGS.name}/${Screen.DISPLAY.name}", StringValue.StringResource(R.string.display_theme), backIcon)
        data object Logs : NavItem("${Screen.SETTINGS.name}/${Screen.LOGS.name}", StringValue.StringResource(R.string.logs), backIcon)
        data object Feedback : NavItem("${Screen.SETTINGS.name}/${Screen.FEEDBACK.name}", StringValue.StringResource(R.string.feedback), backIcon) {
            data object Review : NavItem("${Screen.SETTINGS.name}/${Screen.FEEDBACK.name}/${Screen.REVIEW.name}", StringValue.StringResource(R.string.feedback), backIcon)
        }
        data object Support : NavItem("${Screen.SETTINGS.name}/${Screen.SUPPORT.name}", StringValue.StringResource(R.string.support), backIcon)
        data object Legal : NavItem("${Screen.SETTINGS.name}/${Screen.LEGAL.name}", StringValue.StringResource(R.string.legal), backIcon)
    }
    sealed class Hop {
        data object Entry : NavItem("${Screen.HOP.name}/${HopType.FIRST.name}", HopType.FIRST.hopTitle(), backIcon)
        data object Exit: NavItem("${Screen.HOP.name}/${HopType.LAST.name}", HopType.LAST.hopTitle(), backIcon)
    }

    companion object {
        val settingsIcon = Icons.Outlined.Settings
        val backIcon = Icons.Filled.ArrowBack
        fun from(route: String?) : NavItem {
            return when (route) {
                Main.route -> Main
                Settings.route -> Settings
                Hop.Entry.route -> Hop.Entry
                Hop.Exit.route -> Hop.Exit
                Settings.Display.route -> Settings.Display
                Settings.Logs.route -> Settings.Logs
                Settings.Support.route -> Settings.Support
                Settings.Feedback.route -> Settings.Feedback
                Settings.Feedback.Review.route -> Settings.Feedback.Review
                Settings.Legal.route -> Settings.Legal
                else -> Main
            }
        }
    }
}
