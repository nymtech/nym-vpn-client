package net.nymtech.nymvpn.ui.screens.settings.geoexclusion

import android.content.ClipData
import android.content.res.Configuration
import androidx.annotation.StringRes
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.toClipEntry
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsArrowIcon
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsTitle
import net.nymtech.nymvpn.ui.screens.settings.geoexclusion.components.RegionsCard
import net.nymtech.nymvpn.ui.screens.settings.geoexclusion.components.Socks5AddressCard
import net.nymtech.nymvpn.ui.screens.settings.geoexclusion.components.WarningCard
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

private const val DEFAULT_PORT = "1081"
private const val LOOPBACK_ADDRESS = "127.0.0.1"
private const val PORT_MIN = 1024
private const val PORT_MAX = 65535
private const val FORBIDDEN_PORT = 1080

@Composable
fun GeoExclusionScreen(appUiState: AppUiState, viewModel: GeoExclusionViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val clipboard = LocalClipboard.current
	val scope = rememberCoroutineScope()
	fun copyToClipboard(text: String) {
		scope.launch {
			clipboard.setClipEntry(ClipData.newPlainText(text, text).toClipEntry())
		}
	}
	val failedToStart by viewModel.failedToStart.collectAsStateWithLifecycle()
	val initialPort = remember(appUiState.vpnConfig.geoExclusionPort) {
		appUiState.vpnConfig.geoExclusionPort.toString()
	}
	var portInput by remember(initialPort) { mutableStateOf(initialPort) }
	var portError: Int? by remember { mutableStateOf(null) }
	var lastValidPort by remember(initialPort) { mutableStateOf(initialPort) }

	GeoExclusionScreen(
		geoExclusionEnabled = appUiState.vpnConfig.geoExclusionEnabled,
		failedToStart = failedToStart,
		portInput = portInput,
		portError = portError,
		proxyAddress = lastValidPort,
		onGeoExclusionEnable = { viewModel.onGeoExclusionEnabled(it) },
		onPortChange = {
			portInput = it
			if (portError != null) portError = null
		},
		onPortCommit = {
			when (val port = portInput.trim().toIntOrNull()) {
				FORBIDDEN_PORT -> {
					portInput = lastValidPort
					portError = R.string.geo_exclusion_error_forbidden_port_text
				}
				null, !in PORT_MIN..PORT_MAX -> {
					portInput = lastValidPort
					portError = R.string.geo_exclusion_error_invalid_port_text
				}
				else -> {
					lastValidPort = portInput.trim()
					portError = null
					viewModel.onGeoExclusionPortChanged(port)
				}
			}
		},
		onCopyAddress = { copyToClipboard("$LOOPBACK_ADDRESS:$lastValidPort") },
		onCopyServer = { copyToClipboard(LOOPBACK_ADDRESS) },
		onSetupClick = {
			navController.navigate(Route.Setup)
		},
	)
}

@Composable
fun GeoExclusionScreen(
	geoExclusionEnabled: Boolean,
	failedToStart: Boolean,
	portInput: String,
	@StringRes portError: Int?,
	proxyAddress: String,
	onGeoExclusionEnable: (Boolean) -> Unit,
	onPortChange: (String) -> Unit,
	onPortCommit: () -> Unit,
	onCopyAddress: () -> Unit,
	onCopyServer: () -> Unit = {},
	onSetupClick: () -> Unit,
) {
	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(14.dp.scaledHeight(), Alignment.Top),
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(rememberScrollState())
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth())
			.navigationBarsPadding(),
	) {
		if (failedToStart) {
			WarningCard(stringResource(R.string.geo_exclusion_error_failed_start_text))
		}

		SettingsGroup(
			items = listOf(
				SelectionItem(
					trailing = {
						ScaledSwitch(
							checked = geoExclusionEnabled,
							onClick = { onGeoExclusionEnable(it) },
						)
					},
					title = {
						SettingsTitle(stringResource(R.string.geo_exclusion_enable_title))
					},
				),
			),
		)

		if (geoExclusionEnabled) {
			WarningCard(stringResource(R.string.geo_exclusion_traffic_bypass_text))

			Socks5AddressCard(
				onCopyServer = onCopyServer,
				proxyAddress = proxyAddress,
				onCopy = onCopyAddress,
				portInput = portInput,
				portError = portError,
				onPortChange = onPortChange,
				onPortCommit = onPortCommit,
			)

			Text(
				text = stringResource(R.string.geo_exclusion_excluded_regions_label),
				color = MaterialTheme.colorScheme.primary,
				style = MaterialTheme.typography.labelSmall,
			)
			RegionsCard(
				onRegionClick = {},
				onAddRegionClick = {},
			)
			SettingsGroup(
				items = listOf(
					SelectionItem(
						trailing = {
							SettingsArrowIcon()
						},
						title = {
							SettingsTitle(
								stringResource(R.string.setup_instructions_title),
							)
						},
						onClick = onSetupClick,
					),
				),
			)
		} else {
			Card(
				modifier = Modifier
					.fillMaxWidth()
					.wrapContentHeight(),
				shape = RoundedCornerShape(14.dp),
				colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
			) {
				Text(
					text = stringResource(R.string.geo_exclusion_description),
					color = MaterialTheme.colorScheme.onBackground,
					style = MaterialTheme.typography.bodyMedium,
					modifier = Modifier.padding(16.dp),
				)
			}

			Text(
				text = stringResource(R.string.geo_exclusion_beta_text),
				color = MaterialTheme.colorScheme.onBackground,
				style = MaterialTheme.typography.bodySmall,
			)
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewGeoExclusionScreenOff() {
	NymVPNTheme(Theme.default()) {
		GeoExclusionScreen(
			geoExclusionEnabled = false,
			failedToStart = false,
			portInput = DEFAULT_PORT,
			portError = null,
			proxyAddress = "$LOOPBACK_ADDRESS:$DEFAULT_PORT",
			onGeoExclusionEnable = {},
			onPortChange = {},
			onPortCommit = {},
			onCopyAddress = {},
			onSetupClick = {},
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewGeoExclusionScreenOn() {
	NymVPNTheme(Theme.default()) {
		GeoExclusionScreen(
			geoExclusionEnabled = true,
			failedToStart = false,
			portInput = DEFAULT_PORT,
			portError = null,
			proxyAddress = DEFAULT_PORT,
			onGeoExclusionEnable = {},
			onPortChange = {},
			onPortCommit = {},
			onCopyAddress = {},
			onSetupClick = {},
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewGeoExclusionScreenFailed() {
	NymVPNTheme(Theme.default()) {
		GeoExclusionScreen(
			geoExclusionEnabled = false,
			failedToStart = true,
			portInput = DEFAULT_PORT,
			portError = null,
			proxyAddress = DEFAULT_PORT,
			onGeoExclusionEnable = {},
			onPortChange = {},
			onPortCommit = {},
			onCopyAddress = {},
			onSetupClick = {},
		)
	}
}
