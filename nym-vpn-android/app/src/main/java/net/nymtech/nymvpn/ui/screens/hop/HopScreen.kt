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
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
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
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withLink
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
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
import net.nymtech.nymvpn.util.extensions.getScoreIcon
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.nymvpn.util.extensions.scoreSorted
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib.GatewayType
import java.util.Locale

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HopScreen(gatewayLocation: GatewayLocation, appViewModel: AppViewModel, appUiState: AppUiState, viewModel: HopViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val navController = LocalNavController.current
	val context = LocalContext.current

	var refreshing by remember { mutableStateOf(false) }
	var selectedGateway by remember { mutableStateOf<NymGateway?>(null) }
	var showGatewayDetailsModal by remember { mutableStateOf(false) }
	var showLocationTooltip by remember { mutableStateOf(false) }
	val pullRefreshState = rememberPullToRefreshState()

	val gatewayType = remember {
		when (appUiState.settings.vpnMode) {
			Tunnel.Mode.FIVE_HOP_MIXNET -> {
				when (gatewayLocation) {
					GatewayLocation.EXIT -> GatewayType.MIXNET_EXIT
					GatewayLocation.ENTRY -> GatewayType.MIXNET_ENTRY
				}
			}
			Tunnel.Mode.TWO_HOP_MIXNET -> GatewayType.WG
		}
	}

	val selectedKey = remember {
		when (gatewayLocation) {
			GatewayLocation.ENTRY -> appUiState.entryPointId
			GatewayLocation.EXIT -> appUiState.exitPointId
		}
	}

	val initialGateways = remember {
		when (gatewayType) {
			GatewayType.MIXNET_ENTRY -> appUiState.gateways.entryGateways
			GatewayType.MIXNET_EXIT -> appUiState.gateways.exitGateways
			GatewayType.WG -> appUiState.gateways.wgGateways
		}
	}

	LaunchedEffect(gatewayType, initialGateways) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle(stringResource(if (gatewayLocation == GatewayLocation.EXIT) R.string.exit else R.string.entry)) },
				leading = { NavIcon(Icons.AutoMirrored.Filled.ArrowBack) { navController.popBackStack() } },
				trailing = { NavIcon(Icons.Outlined.Info) { showLocationTooltip = true } },
			),
		)
		viewModel.initializeGateways(initialGateways)
		viewModel.updateCountryCache(gatewayType)
	}

	LaunchedEffect(refreshing) {
		if (refreshing) viewModel.updateCountryCache(gatewayType)
		refreshing = false
	}

	Modal(show = showLocationTooltip, onDismiss = { showLocationTooltip = false }, title = {
		Text(stringResource(R.string.gateway_locations_title), style = CustomTypography.labelHuge)
	}, text = {
		ServerDetailsModalBody(onClick = { context.openWebUrl(context.getString(R.string.location_support_link)) })
	})

	if (showGatewayDetailsModal) {
		selectedGateway?.let {
			GatewayDetailsModal(it, gatewayType, {
				selectedGateway = null
				showGatewayDetailsModal = false
			})
		}
	}

	PullToRefreshBox(
		state = pullRefreshState,
		isRefreshing = refreshing,
		onRefresh = { refreshing = true },
		modifier = Modifier.fillMaxSize(),
	) {
		LazyColumn(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.Top,
			modifier = Modifier
				.fillMaxSize()
				.windowInsetsPadding(WindowInsets.navigationBars)
				.imePadding(),
		) {
			item {
				Column(
					verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight()),
					modifier = Modifier
						.padding(horizontal = 24.dp.scaledWidth())
						.padding(top = 24.dp.scaledHeight()),
				) {
					CustomTextField(
						value = uiState.query,
						onValueChange = { viewModel.onQueryChange(it) },
						modifier = Modifier
							.fillMaxWidth()
							.height(56.dp.scaledHeight())
							.background(Color.Transparent, RoundedCornerShape(30.dp)),
						placeholder = { Text(stringResource(R.string.search_country), color = MaterialTheme.colorScheme.outline) },
						singleLine = true,
						leading = { Icon(Icons.Rounded.Search, contentDescription = stringResource(R.string.search), modifier = Modifier.size(iconSize)) },
						label = { Text(stringResource(R.string.search)) },
						textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onSurface),
					)
				}
			}

			if (uiState.countries.isEmpty() && uiState.queriedGateways.isEmpty() && initialGateways.isEmpty()) {
				item {
					Box(
						modifier = Modifier
							.fillMaxWidth()
							.padding(top = 24.dp.scaledHeight())
							.padding(horizontal = 16.dp.scaledWidth()),
						contentAlignment = Alignment.Center,
					) {
						if (uiState.error) {
							Text(
								stringResource(R.string.country_load_failure),
								style = MaterialTheme.typography.bodyMedium.copy(color = CustomColors.error),
								textAlign = TextAlign.Center,
							)
						} else {
							Text(
								stringResource(R.string.loading),
								style = MaterialTheme.typography.bodyMedium,
								textAlign = TextAlign.Center,
							)
						}
					}
				}
			}

			if (uiState.query.isNotBlank() && uiState.countries.isEmpty() && uiState.queriedGateways.isEmpty() && !uiState.error) {
				item {
					Column(
						horizontalAlignment = Alignment.CenterHorizontally,
						verticalArrangement = Arrangement.spacedBy(5.dp.scaledHeight()),
						modifier = Modifier
							.fillMaxWidth()
							.padding(top = 24.dp.scaledHeight())
							.padding(horizontal = 16.dp.scaledWidth()),
					) {
						Text(
							stringResource(R.string.no_results_found),
							textAlign = TextAlign.Center,
							style = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onBackground),
						)
						Text(
							buildAnnotatedString {
								append(stringResource(R.string.try_another_server_name))
								append(" ")
								withLink(LinkAnnotation.Url(stringResource(R.string.contact_url))) {
									append(stringResource(R.string.contact_for_help))
								}
								append(" ")
								append(stringResource(R.string.or_learn))
								append(" ")
								withLink(LinkAnnotation.Url(stringResource(R.string.docs_url))) {
									append(stringResource(R.string.how_to_run_gateway))
								}
							},
							textAlign = TextAlign.Center,
							style = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.outline),
						)
					}
				}
			}

			items(uiState.countries, key = { it.country }) { country ->
				CountryItem(
					country = country,
					gatewayType = gatewayType,
					gateways = when (gatewayType) {
						GatewayType.MIXNET_ENTRY -> appUiState.gateways.entryGateways
						GatewayType.MIXNET_EXIT -> appUiState.gateways.exitGateways
						GatewayType.WG -> appUiState.gateways.wgGateways
						else -> emptyList()
					}.filter { it.twoLetterCountryISO == country.country.lowercase() },
					selectedKey = selectedKey,
					onSelectionChange = { id ->
						viewModel.onSelected(id, gatewayLocation)
						navController.popBackStack()
					},
					onGatewayDetails = { gateway ->
						selectedGateway = gateway
						showGatewayDetailsModal = true
					},
					modifier = Modifier
						.padding(top = if (uiState.countries.indexOf(country) == 0) 24.dp.scaledHeight() else 0.dp)
						.padding(vertical = 4.dp),
				)
			}

			if (uiState.queriedGateways.isNotEmpty()) {
				itemsIndexed(
					uiState.queriedGateways.scoreSorted(appUiState.settings.vpnMode),
					key = { _, gateway -> gateway.identity },
				) { index, gateway ->
					val locale = gateway.twoLetterCountryISO?.let { Locale(it, it) }
					SurfaceSelectionGroupButton(
						listOf(
							SelectionItem(
								onClick = {
									viewModel.onSelected(gateway.identity, gatewayLocation)
									navController.popBackStack()
								},
								leading = {
									val icon = gateway.getScoreIcon(gatewayType)
									Box(modifier = Modifier.padding(horizontal = 16.dp)) {
										Image(
											icon,
											contentDescription = stringResource(R.string.gateway_score),
											modifier = Modifier.size(16.dp),
										)
									}
								},
								trailing = {
									Box(
										modifier = Modifier
											.clickable {
												selectedGateway = gateway
												showGatewayDetailsModal = true
											}
											.fillMaxHeight(),
										contentAlignment = Alignment.Center,
									) {
										Row(
											horizontalArrangement = Arrangement.spacedBy(16.dp),
											verticalAlignment = Alignment.CenterVertically,
											modifier = Modifier.padding(end = 16.dp),
										) {
											VerticalDivider(modifier = Modifier.height(42.dp))
											Icon(
												Icons.Outlined.Info,
												contentDescription = stringResource(R.string.info),
												modifier = Modifier.size(iconSize),
											)
										}
									}
								},
								title = {
									Text(
										gateway.name,
										maxLines = 1,
										overflow = TextOverflow.Ellipsis,
										style = MaterialTheme.typography.bodyLarge,
									)
								},
								description = {
									Text(
										"${locale?.displayCountry ?: stringResource(R.string.unknown)}, ${gateway.identity}",
										maxLines = 1,
										overflow = TextOverflow.Ellipsis,
										style = MaterialTheme.typography.bodySmall,
									)
								},
								selected = selectedKey == gateway.identity,
							),
						),
						shape = RectangleShape,
						background = MaterialTheme.colorScheme.background,
						divider = false,
						anchorsPadding = 0.dp,
						modifier = Modifier
							.padding(top = if (index == 0 && uiState.countries.isEmpty()) 24.dp.scaledHeight() else 0.dp),
					)
				}
			}
		}
	}
}

@Composable
fun CountryItem(
	country: Locale,
	gatewayType: GatewayType,
	gateways: List<NymGateway>,
	selectedKey: String?,
	onSelectionChange: (String) -> Unit,
	onGatewayDetails: (NymGateway) -> Unit,
	modifier: Modifier = Modifier,
) {
	val context = LocalContext.current
	var expanded by rememberSaveable(key = "expanded_${country.country}") {
		mutableStateOf(gateways.any { it.identity == selectedKey })
	}
	val rotationAngle by animateFloatAsState(targetValue = if (expanded) 180f else 0f)
	val countryCode = country.country.lowercase()

	Column(modifier = modifier) {
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					onClick = { onSelectionChange(countryCode) },
					leading = {
						val icon = ImageVector.vectorResource(context.getFlagImageVectorByName(countryCode))
						Box(modifier = Modifier.padding(horizontal = 16.dp)) {
							Image(
								icon,
								contentDescription = stringResource(R.string.country_flag, country.displayCountry),
								modifier = Modifier.size(iconSize),
							)
						}
					},
					trailing = {
						Box(
							modifier = Modifier
								.clickable { expanded = !expanded }
								.fillMaxHeight(),
							contentAlignment = Alignment.Center,
						) {
							Row(
								horizontalArrangement = Arrangement.spacedBy(16.dp),
								verticalAlignment = Alignment.CenterVertically,
								modifier = Modifier.padding(end = 16.dp),
							) {
								VerticalDivider(modifier = Modifier.height(42.dp))
								Icon(
									Icons.Filled.ArrowDropDown,
									contentDescription = stringResource(if (expanded) R.string.collapse else R.string.expand),
									modifier = Modifier.graphicsLayer(rotationZ = rotationAngle).size(iconSize),
								)
							}
						}
					},
					title = { Text(country.displayCountry, style = MaterialTheme.typography.bodyLarge) },
					description = {
						Text(
							"${gateways.size} ${stringResource(R.string.servers)}",
							style = MaterialTheme.typography.bodySmall,
						)
					},
					selected = countryCode == selectedKey,
				),
			),
			shape = RectangleShape,
			background = MaterialTheme.colorScheme.surface,
			anchorsPadding = 0.dp,
		)

		AnimatedVisibility(
			visible = expanded,
			enter = expandVertically() + fadeIn(),
			exit = shrinkVertically() + fadeOut(),
		) {
			SurfaceSelectionGroupButton(
				gateways.map { gateway ->
					SelectionItem(
						onClick = { onSelectionChange(gateway.identity) },
						leading = {
							val icon = gateway.getScoreIcon(gatewayType)
							Box(modifier = Modifier.padding(horizontal = 16.dp)) {
								Image(
									icon,
									contentDescription = stringResource(R.string.gateway_score),
									modifier = Modifier.size(16.dp),
								)
							}
						},
						trailing = {
							Box(
								modifier = Modifier
									.clickable { onGatewayDetails(gateway) }
									.fillMaxHeight(),
								contentAlignment = Alignment.Center,
							) {
								Row(
									horizontalArrangement = Arrangement.spacedBy(16.dp),
									verticalAlignment = Alignment.CenterVertically,
									modifier = Modifier.padding(end = 16.dp),
								) {
									VerticalDivider(modifier = Modifier.height(42.dp))
									Icon(
										Icons.Outlined.Info,
										contentDescription = stringResource(R.string.info),
										modifier = Modifier.size(iconSize),
									)
								}
							}
						},
						title = {
							Text(
								gateway.name,
								maxLines = 1,
								overflow = TextOverflow.Ellipsis,
								style = MaterialTheme.typography.bodyLarge,
							)
						},
						description = {
							Text(
								gateway.identity,
								maxLines = 1,
								overflow = TextOverflow.Ellipsis,
								style = MaterialTheme.typography.bodySmall,
							)
						},
						selected = selectedKey == gateway.identity,
					)
				},
				shape = RectangleShape,
				background = MaterialTheme.colorScheme.background,
				divider = false,
				anchorsPadding = 0.dp,
			)
		}
	}
}
