package net.nymtech.nymvpn.ui.screens.details

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.details.components.DetailsSectionBottom
import net.nymtech.nymvpn.ui.screens.details.components.DetailsSectionIP
import net.nymtech.nymvpn.ui.screens.details.components.DetailsSectionIdentity
import net.nymtech.nymvpn.ui.screens.details.components.DetailsSectionPerformance
import net.nymtech.nymvpn.ui.screens.details.components.DetailsSectionPrivacy
import net.nymtech.nymvpn.ui.screens.details.components.DetailsTopSection
import net.nymtech.nymvpn.ui.screens.server.GatewayLocation
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.topBorder
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.AsnKind
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.Score

@Composable
fun DetailsScreen(appUiState: AppUiState, id: String, gatewayLocation: String, viewModel: DetailsViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val location = GatewayLocation.valueOf(gatewayLocation)
	val gatewayType = remember {
		when (appUiState.vpnConfig.mode) {
			Tunnel.Mode.FIVE_HOP_MIXNET -> {
				when (location) {
					GatewayLocation.EXIT -> GatewayType.MIXNET_EXIT
					GatewayLocation.ENTRY -> GatewayType.MIXNET_ENTRY
				}
			}
			Tunnel.Mode.TWO_HOP_MIXNET -> GatewayType.WG
		}
	}
	val initialGateways = remember {
		when (gatewayType) {
			GatewayType.MIXNET_ENTRY -> appUiState.gateways.entryGateways
			GatewayType.MIXNET_EXIT -> appUiState.gateways.exitGateways
			GatewayType.WG -> appUiState.gateways.wgGateways
		}
	}
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	LaunchedEffect(gatewayType, initialGateways) {
		viewModel.filterGateways(id, initialGateways, location)
	}
	DetailsScreen(
		detailsUiState = uiState,
		isQuicEnabledLocally = appUiState.settings.quicEnabled,
		gatewayType = gatewayType,
		onSelectServerClick = {
			viewModel.onSelected(uiState.identity, location)
			navController.navigateAndForget(Route.Main())
		},
		onEnableQuicProtocolClick = {
			navController.navigate(Route.Censorship)
		},
		onToggleFavorite = {
			viewModel.onToggleFavorite()
		},
	)
}

@Composable
fun DetailsScreen(
	detailsUiState: DetailsUiState,
	isQuicEnabledLocally: Boolean,
	gatewayType: GatewayType,
	onSelectServerClick: () -> Unit,
	onEnableQuicProtocolClick: () -> Unit,
	onToggleFavorite: () -> Unit,
) {
	val performanceScore = when (gatewayType) {
		GatewayType.MIXNET_ENTRY, GatewayType.MIXNET_EXIT -> detailsUiState.mixnetScore
		GatewayType.WG -> detailsUiState.score
	}
	Column(
		modifier = Modifier
			.fillMaxSize()
			.background(MaterialTheme.colorScheme.background),
	) {
		Column(
			verticalArrangement = Arrangement.spacedBy(16.dp.scaledHeight()),
			modifier = Modifier
				.fillMaxWidth()
				.weight(1f)
				.verticalScroll(rememberScrollState())
				.padding(vertical = 20.dp, horizontal = 16.dp),
		) {
			DetailsTopSection(
				name = detailsUiState.name,
				countryCode = detailsUiState.countryCode,
				location = detailsUiState.location,
				description = detailsUiState.description,
				isFavorite = detailsUiState.isFavorite,
				onToggleFavorite = onToggleFavorite,
			)
			DetailsSectionPrivacy(
				asnKind = detailsUiState.asnKind,
				isQuicSupportedByGateway = detailsUiState.isQuickSupportedByGateway,
				isPostQuantumEnabled = detailsUiState.isPostQuantumEnabled,
				nodeFamilyName = detailsUiState.nodeFamilyName,
				isQuicEnabledLocally = isQuicEnabledLocally,
				onEnableQuicProtocolClick = onEnableQuicProtocolClick,
			)
			DetailsSectionPerformance(performanceScore, detailsUiState.load, detailsUiState.uptime, detailsUiState.lastUpdated)
			DetailsSectionIP(detailsUiState.exitIpv4, detailsUiState.exitIpv6, detailsUiState.asn, detailsUiState.asnName)
			DetailsSectionIdentity(detailsUiState.identity, detailsUiState.buildVersion)
			DetailsSectionBottom(detailsUiState.identity)
		}

		Box(
			modifier = Modifier
				.shadow(elevation = 20.dp, spotColor = Color(0x26000000), ambientColor = Color(0x26000000))
				.topBorder(height = 1.dp, color = MaterialTheme.colorScheme.outline)
				.background(MaterialTheme.colorScheme.surface)
				.navigationBarsPadding()
				.padding(24.dp),
		) {
			MainStyledButton(
				onClick = {
					onSelectServerClick()
				},
				content = {
					Text(
						stringResource(R.string.details_select_server_button),
						style = MaterialTheme.typography.titleMedium,
						color = MaterialTheme.colorScheme.onPrimary,
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(48.dp.scaledHeight()),
				shape = RoundedCornerShape(12.dp),
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
			description = "Enabling safety and privacy in the age of AI and quantum computing." +
				" Follow service status announcements at https://t.me/oceanusp17o",
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
			isQuickSupportedByGateway = true,
			isPostQuantumEnabled = true,
			nodeFamilyName = "Nym Family",
		)
		DetailsScreen(
			detailsUiState = detailsUiState,
			isQuicEnabledLocally = false,
			gatewayType = GatewayType.WG,
			onSelectServerClick = {},
			onEnableQuicProtocolClick = {},
			onToggleFavorite = {},
		)
	}
}
