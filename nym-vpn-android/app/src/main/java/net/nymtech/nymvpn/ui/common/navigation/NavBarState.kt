package net.nymtech.nymvpn.ui.common.navigation

import androidx.annotation.StringRes
import net.nymtech.nymvpn.ui.screens.main.profiles.Profile

sealed class NavBarState {
	data object Hidden : NavBarState()
	data object Empty : NavBarState()

	class Main(val selectedProfile: Profile?, val onProfileClick: () -> Unit, val onProfileSelect: (Profile) -> Unit, val onSettingsClick: () -> Unit) : NavBarState()

	class WithClose(@StringRes val titleRes: Int?, val showClose: Boolean = true, val onClose: () -> Unit = {}) : NavBarState()

	class WithBack(@StringRes val titleRes: Int?, val onBack: (() -> Unit)?, val trailing: Trailing = Trailing.None) : NavBarState()

	sealed class Trailing {
		data object None : Trailing()
		class Info(val onClick: () -> Unit) : Trailing()
		class LogsMenu(val onDownload: () -> Unit, val onShare: () -> Unit, val onDelete: () -> Unit) : Trailing()
	}
}
