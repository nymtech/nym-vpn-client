package net.nymtech.nymvpn.ui.screens.settings.appearance.display

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.IconSurfaceButton
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun DisplayScreen(appUiState: AppUiState, viewModel: DisplayViewModel = hiltViewModel()) {

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier =
		Modifier
			.fillMaxSize()
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		IconSurfaceButton(
			title = stringResource(R.string.automatic),
			description = stringResource(R.string.device_theme),
			onClick = {
				viewModel.onThemeChange(Theme.AUTOMATIC)
			},
			selected = appUiState.settings.theme == Theme.AUTOMATIC,
		)
		IconSurfaceButton(
			title = stringResource(R.string.light_theme),
			onClick = { viewModel.onThemeChange(Theme.LIGHT_MODE) },
			selected = appUiState.settings.theme == Theme.LIGHT_MODE,
		)
		IconSurfaceButton(
			title = stringResource(R.string.dark_theme),
			onClick = { viewModel.onThemeChange(Theme.DARK_MODE) },
			selected = appUiState.settings.theme == Theme.DARK_MODE,
		)
	}
}
