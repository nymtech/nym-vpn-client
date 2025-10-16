package net.nymtech.nymvpn.ui.screens.details

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.details.components.CountryFlag
import net.nymtech.nymvpn.ui.screens.details.components.DetailsSectionBottom
import net.nymtech.nymvpn.ui.screens.details.components.DetailsSectionIP
import net.nymtech.nymvpn.ui.screens.details.components.DetailsSectionIdentity
import net.nymtech.nymvpn.ui.screens.details.components.DetailsSectionPerformance
import net.nymtech.nymvpn.ui.screens.details.components.DetailsSectionPrivacy
import net.nymtech.nymvpn.ui.screens.hop.GatewayLocation
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.topBorder
import nym_vpn_lib_types.AsnKind
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.Score

@Composable
fun DetailsScreen(appUiState: AppUiState, id: String, type: GatewayType, gatewayLocation: String, viewModel: DetailsViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val location = GatewayLocation.valueOf(gatewayLocation)
	val initialGateways = remember {
		when (type) {
			GatewayType.MIXNET_ENTRY -> appUiState.gateways.entryGateways
			GatewayType.MIXNET_EXIT -> appUiState.gateways.exitGateways
			GatewayType.WG -> appUiState.gateways.wgGateways
		}
	}
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	LaunchedEffect(type, initialGateways) {
		viewModel.filterGateways(id, initialGateways)
	}
	DetailsScreen(
		detailsUiState = uiState,
		onSelectServerClick = {
			viewModel.onSelected(uiState.identity, location)
			navController.navigateAndForget(Route.Main())
		},
	)
}

@Composable
fun DetailsScreen(detailsUiState: DetailsUiState, onSelectServerClick: () -> Unit) {
	Column(
		verticalArrangement = Arrangement.spacedBy(8.dp.scaledHeight(), Alignment.Top),
		horizontalAlignment = Alignment.Start,
		modifier = Modifier
			.fillMaxSize()
			.background(MaterialTheme.colorScheme.background)
			.padding(WindowInsets.systemBars.asPaddingValues()),
	) {
		Column(
			modifier = Modifier
				.fillMaxWidth()
				.background(MaterialTheme.colorScheme.background)
				.weight(1f)
				.verticalScroll(rememberScrollState())
				.padding(24.dp),
		) {
			Text(
				text = detailsUiState.name,
				style = CustomTypography.titleMediumPlus,
				color = MaterialTheme.colorScheme.onBackground,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
			Row(
				modifier = Modifier
					.padding(top = 16.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				CountryFlag(detailsUiState.countryCode, 16.dp)
				Spacer(modifier = Modifier.width(8.dp))
				Text(
					text = detailsUiState.location,
					style = Typography.titleMedium,
					color = MaterialTheme.colorScheme.onBackground,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)
			}
			detailsUiState.description?.let {
				Text(
					modifier = Modifier.padding(top = 16.dp),
					text = it,
					style = Typography.bodyMedium,
					color = MaterialTheme.colorScheme.outline,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular))
				)
			}
			DetailsSectionPrivacy(detailsUiState.asnKind)
			DetailsSectionPerformance(detailsUiState.score, detailsUiState.load, detailsUiState.uptime, detailsUiState.lastUpdated)
			DetailsSectionIP(detailsUiState.exitIpv4, detailsUiState.exitIpv6, detailsUiState.asn, detailsUiState.asnName)
			DetailsSectionIdentity(detailsUiState.identity, detailsUiState.buildVersion)
			DetailsSectionBottom(detailsUiState.identity)
		}

		Box(
			modifier = Modifier
				.shadow(elevation = 20.dp, spotColor = Color(0x26000000), ambientColor = Color(0x26000000))
				.topBorder(height = 1.dp, color = MaterialTheme.colorScheme.outline)
				.background(MaterialTheme.colorScheme.surface)
				.padding(24.dp),
		) {
			MainStyledButton(
				onClick = {
					onSelectServerClick()
				},
				content = {
					Text(
						stringResource(R.string.details_select_server_button),
						style = Typography.titleMedium,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				},
				color = MaterialTheme.colorScheme.primary,
				modifier = Modifier
					.fillMaxWidth()
					.height(42.dp.scaledHeight()),
			)
		}
	}
}

@Composable
@PreviewLightDark
internal fun PreviewPrivacyScreen() {
	NymVPNTheme(Theme.default()) {
		val detailsUiState = DetailsUiState(
			identity = "wqewqewqewqewqfade2123123",
			name = "Jacksonville-Cloak04",
			description = "Enabling safety and privacy in the age of AI and quantum computing. Follow service status announcements at https://t.me/oceanusp17o",
			location = "Jacksonville, Texas, United States",
			countryCode = "DE",
			mixnetScore = Score.HIGH,
			score = Score.HIGH,
			load = Score.HIGH,
			uptime = 89f,
			lastUpdated = "September 11, 2025 at 13:31",
			asnName = "Google LLC",
			asn = "AS29234",
			asnKind = AsnKind.RESIDENTIAL,
			buildVersion = "1.2.4",
			exitIpv4 = "12.34.152.125",
			exitIpv6 = "12:ff:14::155",
		)
		DetailsScreen(detailsUiState = detailsUiState, onSelectServerClick = {})
	}
}
