package net.nymtech.nymvpn.ui.screens.settings.display

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
import net.nymtech.nymvpn.ui.common.buttons.RadioSurfaceButton
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.screenPadding

@Composable
fun DisplayScreen(viewModel: DisplayViewModel = hiltViewModel()) {

    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    Column(
        horizontalAlignment = Alignment.Start,
        verticalArrangement = Arrangement.spacedBy(screenPadding, Alignment.Top),
        modifier = Modifier.fillMaxSize().padding(top = screenPadding).padding(horizontal = screenPadding)) {
        RadioSurfaceButton(
            title = stringResource(R.string.automatic),
            description = stringResource(R.string.device_theme),
            onClick = { viewModel.onThemeChange(Theme.AUTOMATIC) },
            selected = uiState.theme == Theme.AUTOMATIC)
        RadioSurfaceButton(
            title = stringResource(R.string.light_theme),
            onClick = { viewModel.onThemeChange(Theme.LIGHT_MODE) },
            selected = uiState.theme == Theme.LIGHT_MODE)
        RadioSurfaceButton(
            title = stringResource(R.string.dark_theme),
            onClick = { viewModel.onThemeChange(Theme.DARK_MODE) },
            selected = uiState.theme == Theme.DARK_MODE)
    }
}