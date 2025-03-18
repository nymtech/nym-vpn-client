package net.nymtech.nymvpn.ui.screens.settings.appearance.display

import android.os.Build
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.buttons.IconSurfaceButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun DisplayScreen(appUiState: AppUiState, appViewModel: AppViewModel, viewModel: DisplayViewModel = hiltViewModel()) {
	val navController = LocalNavController.current

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle(stringResource(R.string.display_theme)) },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			),
		)
	}

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier =
		Modifier
			.fillMaxSize()
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		enumValues<Theme>().forEach {
			val title = when (it) {
				Theme.DARK_MODE -> stringResource(R.string.dark_theme)
				Theme.LIGHT_MODE -> stringResource(R.string.light_theme)
				Theme.AUTOMATIC -> stringResource(R.string.automatic)
				Theme.DYNAMIC -> stringResource(R.string.dynamic)
			}
			val description = when (it) {
				Theme.AUTOMATIC -> stringResource(R.string.device_theme)
				Theme.DYNAMIC -> stringResource(R.string.system_wallpaper)
				else -> null
			}
			if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S && it == Theme.DYNAMIC) {
				return@Column
			}
			IconSurfaceButton(
				title = title,
				description = description,
				onClick = {
					viewModel.onThemeChange(it)
				},
				selected = appUiState.settings.theme == it,
			)
		}
	}
}
