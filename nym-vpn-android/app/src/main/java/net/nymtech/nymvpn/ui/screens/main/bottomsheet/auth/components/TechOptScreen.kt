package net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun TechOptView(
	statsEnabled: Boolean,
	sentryEnabled: Boolean,
	onNetworkStatsEnable: (enabled: Boolean) -> Unit,
	onMonitoringEnable: (enabled: Boolean) -> Unit,
	onContinueClick: () -> Unit,
	modifier: Modifier = Modifier,
) {
	Column(
		modifier = modifier
			.background(MaterialTheme.colorScheme.surface)
			.fillMaxWidth()
			.padding(horizontal = 18.dp, vertical = 16.dp),
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(22.dp),
	) {
		Icon(
			imageVector = ImageVector.vectorResource(R.drawable.app_label),
			contentDescription = stringResource(R.string.app_name),
			tint = MaterialTheme.colorScheme.onPrimaryContainer,
		)

		Text(
			text = stringResource(R.string.welcome_title),
			style = MaterialTheme.typography.titleLarge,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
		Text(
			text = stringResource(R.string.welcome_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onBackground,
			textAlign = TextAlign.Center,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Bottom),
			modifier = Modifier.padding(vertical = 24.dp.scaledHeight()),
		) {
			SettingsSection(statsEnabled, sentryEnabled, onNetworkStatsEnable, onMonitoringEnable)
			Column(
				horizontalAlignment = Alignment.CenterHorizontally,
				modifier = Modifier.padding(top = 8.dp),
			) {
				MainStyledButton(
					onClick = onContinueClick,
					content = {
						Text(
							text = stringResource(R.string.welcome_continue),
							style = MaterialTheme.typography.titleMedium,
						)
					},
					color = MaterialTheme.colorScheme.primary,
					modifier = Modifier
						.fillMaxWidth()
						.height(48.dp.scaledHeight()),
					shape = RoundedCornerShape(12.dp),
				)
			}
		}
	}
}

@Preview(name = "TechOptScreenPreview", uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun PreviewTechOptScreenViewDark() {
	NymVPNTheme(Theme.DARK_MODE) {
		TechOptView(
			statsEnabled = true,
			sentryEnabled = true,
			onMonitoringEnable = {},
			onNetworkStatsEnable = {},
			onContinueClick = {},
		)
	}
}
