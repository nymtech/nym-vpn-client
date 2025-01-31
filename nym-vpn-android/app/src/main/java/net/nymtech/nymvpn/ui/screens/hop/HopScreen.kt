package net.nymtech.nymvpn.ui.screens.hop

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.ArrowDropDown
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
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
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.core.os.ConfigurationCompat
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.VerticalDivider
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.getFlagImageVectorByName
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.GatewayType
import java.text.Collator
import java.util.Locale

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HopScreen(gatewayLocation: GatewayLocation, appViewModel: AppViewModel, appUiState: AppUiState, viewModel: HopViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val context = LocalContext.current
	val navController = LocalNavController.current

	var refreshing by remember { mutableStateOf(false) }
	val pullRefreshState = rememberPullToRefreshState()

	val currentLocale = ConfigurationCompat.getLocales(context.resources.configuration)[0]
	val collator = Collator.getInstance(currentLocale)

	var showLocationTooltip by remember { mutableStateOf(false) }

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = {
					NavTitle(
						when (gatewayLocation) {
							GatewayLocation.EXIT -> stringResource(R.string.exit_location)
							GatewayLocation.ENTRY -> stringResource(R.string.entry_location)
						},
					)
				},
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack) {
						navController.popBackStack()
					}
				},
				trailing = {
					NavIcon(Icons.Outlined.Info) {
						showLocationTooltip = true
					}
				},
			),
		)
	}

	val gatewayType = when (appUiState.settings.vpnMode) {
		Tunnel.Mode.FIVE_HOP_MIXNET -> {
			when (gatewayLocation) {
				GatewayLocation.EXIT -> GatewayType.MIXNET_EXIT
				GatewayLocation.ENTRY -> GatewayType.MIXNET_ENTRY
			}
		}
		Tunnel.Mode.TWO_HOP_MIXNET -> GatewayType.WG
	}

	val gateways = when (gatewayType) {
		GatewayType.MIXNET_ENTRY -> appUiState.gateways.entryGateways
		GatewayType.MIXNET_EXIT -> appUiState.gateways.exitGateways
		GatewayType.WG -> appUiState.gateways.wgGateways
	}

// 	val selectedCountry = when (gatewayLocation) {
// 		GatewayLocation.EXIT -> appUiState.exitCountry
// 		GatewayLocation.ENTRY -> appUiState.entryCountry
// 	}

	val queriedCountries =
		remember(uiState.queriedCountries) {
			uiState.queriedCountries.sortedWith(compareBy(collator) { it.identity })
		}

	val allCountries = remember(gateways) {
		gateways.sortedWith(compareBy(collator) { it.identity })
	}

	val displayGateways = if (uiState.query.isBlank()) allCountries else queriedCountries

	LaunchedEffect(Unit) {
		viewModel.updateCountryCache(gatewayType)
	}

	LaunchedEffect(refreshing) {
		if (refreshing) viewModel.updateCountryCache(gatewayType)
		refreshing = false
	}

	fun onSelectionChange(id: String) {
		viewModel.onSelected(id, gatewayLocation)
		navController.navigateAndForget(Route.Main())
	}

	Modal(show = showLocationTooltip, onDismiss = { showLocationTooltip = false }, title = {
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

	PullToRefreshBox(
		state = pullRefreshState,
		isRefreshing = refreshing,
		onRefresh = { refreshing = true },
	) {
		LazyColumn(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.Top,
			modifier =
			Modifier
				.fillMaxSize().windowInsetsPadding(WindowInsets.navigationBars),
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
							viewModel.onQueryChange(it, gateways)
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
			if (gateways.isEmpty()) {
				item {
					if (uiState.error) {
						Text(
							stringResource(id = R.string.country_load_failure),
							style = MaterialTheme.typography.bodyMedium.copy(
								color = CustomColors.error,
							),
						)
					} else {
						Text(
							stringResource(id = R.string.loading),
							style = MaterialTheme.typography.bodyMedium,
						)
					}
				}
			}
			if (gateways.isNotEmpty()) {
				item {
// 				if (gatewayLocation == GatewayLocation.ENTRY) {
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
// 						stringResource(R.string.automatic),
// 						onClick = {
// 							viewModel.onSelected(Country(isLowLatency = true), gatewayLocation)
// 							navController.navigateAndForget(Route.Main())
// 						},
// 						trailing = {
// 							if (selectedCountry.isLowLatency == true) {
// 								SelectedLabel()
// 							}
// 						},
// 					)
// 				}
				}
			}
			items(gateways.distinctBy { it.twoLetterCountryISO }, key = { it.identity }) { country ->
				val locale = country.twoLetterCountryISO?.let { Locale(it, it) }
				Column(modifier = Modifier.padding(bottom = 8.dp)) {
					var expanded by remember { mutableStateOf(false) }
					val rotationAngle by animateFloatAsState(targetValue = if (expanded) 180f else 0f)
					SurfaceSelectionGroupButton(
						listOf(
							SelectionItem(
								onClick = {
									country.twoLetterCountryISO?.let {
										onSelectionChange(it)
									}
								},
								leading = {
									val icon = country.twoLetterCountryISO?.let {
										ImageVector.vectorResource(
											context.getFlagImageVectorByName(
												it.lowercase(),
											),
										)
									} ?: ImageVector.vectorResource(context.getFlagImageVectorByName("unknown"))
									Image(
										icon,
										icon.name,
										modifier =
										Modifier
											.size(
												iconSize,
											),
									)
								},
								trailing = {
									Row(
										horizontalArrangement = Arrangement.spacedBy(16.dp),
										verticalAlignment = Alignment.CenterVertically,
									) {
										VerticalDivider(modifier = Modifier.height(42.dp))
										val icon = Icons.Filled.ArrowDropDown
										Icon(
											imageVector = icon,
											contentDescription = if (expanded) "Collapse" else "Expand",
											modifier = Modifier.graphicsLayer(rotationZ = rotationAngle).clickable {
												expanded = !expanded
											},
										)
									}
								},
								title = { Text(locale?.displayCountry ?: "Unknown", style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
								description = {
									Text(
										gateways.count { it.twoLetterCountryISO == country.twoLetterCountryISO }.toString() + " servers",
										style = MaterialTheme.typography.bodySmall.copy(MaterialTheme.colorScheme.outline),
									)
								},
							),
						),
						shape = RectangleShape,
						background = MaterialTheme.colorScheme.surface,
					)
					AnimatedVisibility(
						visible = expanded,
						enter = expandVertically() + fadeIn(),
						exit = shrinkVertically() + fadeOut(),
					) {
						SurfaceSelectionGroupButton(
							gateways.filter { it.twoLetterCountryISO == country.twoLetterCountryISO }.map { gateway ->
								SelectionItem(
									onClick = {
										onSelectionChange(gateway.identity)
									},
									leading = {
										// TODO this will change, just random threshold for now
										val icon = gateway.mixnetPerformance?.let {
											when (it) {
												in 0u..45u -> ImageVector.vectorResource(R.drawable.bars_1)
												in 46u..75u -> ImageVector.vectorResource(R.drawable.bars_2)
												in 76u..100u -> ImageVector.vectorResource(R.drawable.bars_3)
												else -> ImageVector.vectorResource(R.drawable.bars_2)
											}
										} ?: ImageVector.vectorResource(R.drawable.bars_2)
										Image(
											icon,
											icon.name,
											modifier =
											Modifier.height(16.dp).width(15.dp),
										)
									},
									trailing = {
										Row(
											horizontalArrangement = Arrangement.spacedBy(16.dp),
											verticalAlignment = Alignment.CenterVertically,
										) {
											val icon = Icons.Outlined.Info
											VerticalDivider(modifier = Modifier.height(42.dp))
											Icon(icon, icon.name, Modifier.size(iconSize))
										}
									},
									title = {
										Text(
											"Unknown name",
											maxLines = 1,
											overflow = TextOverflow.Ellipsis,
											style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
										)
									},
									description = {
										Text(
											gateway.identity,
											maxLines = 1,
											overflow = TextOverflow.Ellipsis,
											style = MaterialTheme.typography.bodySmall.copy(MaterialTheme.colorScheme.outline),
										)
									},
								)
							},
							shape = RectangleShape,
							background = MaterialTheme.colorScheme.background,
						)
					}
				}
			}
		}
	}
}
