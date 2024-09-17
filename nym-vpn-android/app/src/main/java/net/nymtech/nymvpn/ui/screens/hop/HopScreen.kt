package net.nymtech.nymvpn.ui.screens.hop

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import androidx.core.os.ConfigurationCompat
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavHostController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Destination
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.SelectionItemButton
import net.nymtech.nymvpn.ui.common.labels.SelectedLabel
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.getFlagImageVectorByName
import net.nymtech.nymvpn.util.extensions.go
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.GatewayType
import java.text.Collator

@Composable
fun HopScreen(
	gatewayLocation: GatewayLocation,
	appViewModel: AppViewModel,
	navController: NavHostController,
	appUiState: AppUiState,
	viewModel: HopViewModel = hiltViewModel(),
) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val context = LocalContext.current

	val currentLocale = ConfigurationCompat.getLocales(context.resources.configuration)[0]
	val collator = Collator.getInstance(currentLocale)

	val gatewayType = when (appUiState.settings.vpnMode) {
		Tunnel.Mode.FIVE_HOP_MIXNET -> {
			when (gatewayLocation) {
				GatewayLocation.EXIT -> GatewayType.MIXNET_EXIT
				GatewayLocation.ENTRY -> GatewayType.MIXNET_ENTRY
			}
		}
		Tunnel.Mode.TWO_HOP_MIXNET -> GatewayType.WG
	}

	val countries = when (gatewayType) {
		GatewayType.MIXNET_ENTRY -> appUiState.gateways.entryCountries
		GatewayType.MIXNET_EXIT -> appUiState.gateways.exitCountries
		GatewayType.WG -> appUiState.gateways.wgCountries
	}

	val selectedCountry = when (gatewayLocation) {
		GatewayLocation.EXIT -> appUiState.settings.lastHopCountry
		GatewayLocation.ENTRY -> appUiState.settings.firstHopCountry
	}

	val queriedCountries =
		remember(uiState.queriedCountries) {
			uiState.queriedCountries.sortedWith(compareBy(collator) { it.name })
		}

	val allCountries = remember(countries) {
		countries.sortedWith(compareBy(collator) { it.name })
	}

	val displayCountries = if (uiState.query.isBlank()) allCountries else queriedCountries

	LaunchedEffect(Unit) {
		viewModel.updateCountryCache(gatewayType)
	}

	Modal(show = appUiState.showLocationTooltip, onDismiss = { appViewModel.onToggleShowLocationTooltip() }, title = {
		Text(
			text = stringResource(R.string.gateway_locations_title),
			color = MaterialTheme.colorScheme.onSurface,
			style = CustomTypography.labelHuge,
		)
	}, text = {
		GatewayModalBody(
			onClick = {
				context.openWebUrl(context.getString(R.string.location_support_link))
			},
		)
	})

	LazyColumn(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.Top,
		modifier =
		Modifier
			.fillMaxSize(),
	) {
		item {
			Column(
				verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight()),
				modifier = Modifier
					.padding(bottom = 24.dp.scaledHeight())
					.padding(horizontal = 24.dp.scaledWidth()),
			) {
				Box(
					modifier =
					Modifier
						.fillMaxWidth()
						.padding(
							horizontal = 16.dp.scaledWidth(),
						),
				)
				var query: String by rememberSaveable { mutableStateOf("") }
				CustomTextField(
					value = query,
					onValueChange = {
						query = it
						viewModel.onQueryChange(it, countries)
					},
					modifier = Modifier
						.fillMaxWidth()
						.height(56.dp.scaledHeight())
						.background(color = Color.Transparent, RoundedCornerShape(30.dp)),
					placeholder = {
						Text(
							stringResource(id = R.string.search_country),
							color = MaterialTheme.colorScheme.outline,
							style = MaterialTheme.typography.bodyLarge,
						)
					},
					singleLine = true,
					leading = {
						val icon = Icons.Rounded.Search
						Icon(
							imageVector = icon,
							modifier = Modifier.size(iconSize),
							tint = MaterialTheme.colorScheme.onBackground,
							contentDescription = icon.name,
						)
					},
					label = {
						Text(
							stringResource(R.string.search),
						)
					},
					textStyle = MaterialTheme.typography.bodyLarge.copy(
						color = MaterialTheme.colorScheme.onSurface,
					),
				)
			}
		}
		item {
			if (countries.isNotEmpty()) {
				// TODO disable for now
// 				val lowLatencyCountry = uiState.lowLatencyCountry
// 				if (lowLatencyCountry != null) {
// 					val name = StringUtils.buildCountryNameString(lowLatencyCountry, context)
// 					val icon = ImageVector.vectorResource(R.drawable.bolt)
// 					SelectionItemButton(
// 						{
// 							Icon(
// 								icon,
// 								icon.name,
// 								modifier =
// 								Modifier
// 									.padding(
// 										horizontal = 24.dp.scaledWidth(),
// 										vertical = 16.dp.scaledHeight(),
// 									)
// 									.size(
// 										iconSize,
// 									),
// 								tint = MaterialTheme.colorScheme.onSurface,
// 							)
// 						},
// 						name,
// 						onClick = {
// 							viewModel.onSelected(lowLatencyCountry)
// 							navController.go(NavItem.Main.route)
// 						},
// 						trailingText =
// 						if (lowLatencyCountry == uiState.selected) {
// 							stringResource(id = R.string.is_selected)
// 						} else {
// 							null
// 						},
// 					)
// 				}
			} else {
				Text(
					stringResource(id = R.string.country_load_failure),
					style = MaterialTheme.typography.bodyMedium.copy(
						color = CustomColors.error,
					),
				)
			}
		}
		items(displayCountries, key = { it.isoCode }) {
			if (it.isLowLatency) return@items
			val icon =
				ImageVector.vectorResource(
					context.getFlagImageVectorByName(
						it.isoCode.lowercase(),
					),
				)
			SelectionItemButton(
				{
					Image(
						icon,
						icon.name,
						modifier =
						Modifier
							.padding(horizontal = 24.dp.scaledWidth(), 16.dp.scaledHeight())
							.size(
								iconSize,
							),
					)
				},
				buttonText = it.name,
				onClick = {
					viewModel.onSelected(it, gatewayLocation)
					navController.go(Destination.Main.createRoute(false))
				},
				trailing = {
					if (it.isoCode == selectedCountry.isoCode) {
						SelectedLabel()
					}
				},
			)
		}
	}
}
