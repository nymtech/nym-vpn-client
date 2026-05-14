package net.nymtech.nymvpn.ui.screens.settings.appearance.display

import android.content.res.Configuration
import android.os.Build
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.IconSurfaceButton
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun DisplayScreen(appUiState: AppUiState, viewModel: DisplayViewModel = hiltViewModel()) {
	DisplayScreen(
		selectedTheme = appUiState.settings.theme ?: Theme.default(),
		onThemeChange = viewModel::onThemeChange,
	)
}

@Composable
fun DisplayScreen(selectedTheme: Theme, onThemeChange: (Theme) -> Unit) {
	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(16.dp.scaledHeight(), Alignment.Top),
		modifier =
		Modifier
			.fillMaxSize()
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 16.dp.scaledWidth()),
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
				onClick = { onThemeChange(it) },
				selected = selectedTheme == it,
			)
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewDisplayScreen() {
	NymVPNTheme(Theme.default()) {
		DisplayScreen(
			selectedTheme = Theme.DARK_MODE,
			onThemeChange = {},
		)
	}
}
