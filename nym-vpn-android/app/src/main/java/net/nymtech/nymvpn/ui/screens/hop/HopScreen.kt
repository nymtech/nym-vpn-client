package net.nymtech.nymvpn.ui.screens.hop

import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
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
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Shuffle
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarEvent
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.screens.hop.components.CountryItem
import net.nymtech.nymvpn.ui.screens.hop.components.ExitServerDetailsModal
import net.nymtech.nymvpn.ui.screens.hop.components.QuicInfoMessage
import net.nymtech.nymvpn.ui.screens.hop.components.ServerDetailsModalBody
import net.nymtech.nymvpn.ui.screens.hop.components.ServerDetailsTrailingContent
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.getScoreIcon
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.isQuicSupported
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.safePopBackStack
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.AsnKind
import nym_vpn_lib_types.GatewayType
import java.util.Locale

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HopScreen(gatewayLocation: GatewayLocation, appUiState: AppUiState, navBarEvent: NavBarEvent?, onNavBarEventConsume: () -> Unit, viewModel: HopViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val navController = LocalNavController.current
	val context = LocalContext.current
	val locationSupportLink = stringResource(R.string.location_support_link)

	var refreshing by remember { mutableStateOf(false) }

	var showLocationTooltip by remember { mutableStateOf(false) }
	var showExitServerTooltip by remember { mutableStateOf(false) }

	LaunchedEffect(navBarEvent, gatewayLocation) {
		when (navBarEvent) {
			NavBarEvent.EntryLocationInfoClicked -> {
				if (gatewayLocation == GatewayLocation.ENTRY) {
					showLocationTooltip = true
					onNavBarEventConsume()
				}
			}
			NavBarEvent.ExitLocationInfoClicked -> {
				if (gatewayLocation == GatewayLocation.EXIT) {
					showExitServerTooltip = true
					onNavBarEventConsume()
				}
			}
			else -> Unit
		}
	}

	ServerDetailsModalBody(
		showLocationTooltip = showLocationTooltip,
		onClick = { context.openWebUrl(locationSupportLink) },
		onDismiss = { showLocationTooltip = false },
	)

	ExitServerDetailsModal(
		showModal = showExitServerTooltip,
		onClick = { context.openWebUrl(it) },
		onDismiss = { showExitServerTooltip = false },
	)

	val gatewayType = remember {
		when (appUiState.vpnConfig.mode) {
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

	val canShowQuicLabel = remember {
		gatewayLocation == GatewayLocation.ENTRY &&
			appUiState.vpnConfig.mode == Tunnel.Mode.TWO_HOP_MIXNET &&
			appUiState.settings.quicEnabled
	}

	LaunchedEffect(gatewayType, initialGateways) {
		viewModel.initializeGateways(initialGateways, gatewayLocation == GatewayLocation.EXIT)
		viewModel.updateCountryCache(gatewayType)
	}

	LaunchedEffect(refreshing) {
		if (refreshing) viewModel.updateCountryCache(gatewayType)
		refreshing = false
	}

	HopScreenContent(
		uiState = uiState,
		selectedKey = selectedKey,
		gatewayType = gatewayType,
		canShowQuicLabel = canShowQuicLabel,
		gatewayLocation = gatewayLocation,
		initialGatewaysEmpty = initialGateways.isEmpty(),
		isRefreshing = refreshing,
		onRefresh = { refreshing = true },
		onQueryChange = { viewModel.onQueryChange(it) },
		onSelect = { id ->
			viewModel.onSelected(id, gatewayLocation)
			navController.safePopBackStack()
		},
		onNavigateToCensorship = { navController.navigate(Route.Censorship) },
		onNavigateToServerDetails = { gateway ->
			navController.goFromRoot(Route.ServerDetails(gateway.identity, gatewayLocation.name))
		},
	)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun HopScreenContent(
	uiState: HopUiState,
	selectedKey: String?,
	gatewayType: GatewayType,
	canShowQuicLabel: Boolean,
	gatewayLocation: GatewayLocation,
	initialGatewaysEmpty: Boolean,
	isRefreshing: Boolean,
	onRefresh: () -> Unit,
	onQueryChange: (String) -> Unit,
	onSelect: (String) -> Unit,
	onNavigateToCensorship: () -> Unit,
	onNavigateToServerDetails: (NymGateway) -> Unit,
) {
	val pullRefreshState = rememberPullToRefreshState()
	val listState = rememberLazyListState()
	var hasScrolled by remember { mutableStateOf(false) }

	LaunchedEffect(uiState.items) {
		if (hasScrolled || selectedKey == null || uiState.items.isEmpty()) return@LaunchedEffect
		val index = uiState.items.indexOfFirst { item ->
			when (item) {
				is ItemType.CountryItem -> {
					val countryCode = item.locale.country.lowercase()
					countryCode == selectedKey ||
						item.gateways.any { it.identity == selectedKey } ||
						item.regions?.any { it.region.equals(selectedKey, true) } == true
				}
				is ItemType.GatewayItem -> item.gateway.identity == selectedKey
			}
		}
		if (index >= 0) {
			listState.animateScrollToItem(index + 2)
			hasScrolled = true
		}
	}

	PullToRefreshBox(
		state = pullRefreshState,
		isRefreshing = isRefreshing,
		onRefresh = onRefresh,
		modifier = Modifier.fillMaxSize(),
	) {
		LazyColumn(
			state = listState,
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
					if (canShowQuicLabel) {
						QuicInfoMessage(onNavigateToQuicSettings = onNavigateToCensorship)
					}
					CustomTextField(
						value = uiState.query,
						onValueChange = onQueryChange,
						modifier = Modifier
							.fillMaxWidth()
							.height(56.dp.scaledHeight())
							.background(Color.Transparent, RoundedCornerShape(30.dp)),
						placeholder = { Text(stringResource(R.string.search_country), color = MaterialTheme.colorScheme.onBackground) },
						singleLine = true,
						leading = { Icon(Icons.Rounded.Search, contentDescription = stringResource(R.string.search), modifier = Modifier.size(iconSize)) },
						label = { Text(stringResource(R.string.search)) },
						showClearIcon = true,
						textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onPrimaryContainer),
					)
				}
			}

			item {
				SurfaceSelectionGroupButton(
					items = listOf(
						SelectionItem(
							onClick = { onSelect("Random") },
							leading = {
								Box(modifier = Modifier.padding(horizontal = 16.dp)) {
									Icon(
										imageVector = Icons.Rounded.Shuffle,
										contentDescription = null,
										modifier = Modifier.size(iconSize),
									)
								}
							},
							title = { Text(stringResource(R.string.random_text), style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onPrimaryContainer) },
							selected = selectedKey == null,
						),
					),
					shape = RectangleShape,
					background = MaterialTheme.colorScheme.surface,
					anchorsPadding = 0.dp,
					modifier = Modifier
						.padding(top = 12.dp.scaledHeight())
						.padding(vertical = 4.dp),
				)
			}

			if (uiState.items.isEmpty() && initialGatewaysEmpty) {
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
								style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.error),
								textAlign = TextAlign.Center,
							)
						} else {
							Text(
								stringResource(R.string.loading),
								style = MaterialTheme.typography.bodyMedium,
								textAlign = TextAlign.Center,
								color = MaterialTheme.colorScheme.onBackground,
							)
						}
					}
				}
			}

			if (uiState.query.isNotBlank() && uiState.items.isEmpty() && !uiState.error) {
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
								withStyle(
									style = SpanStyle(
										color = MaterialTheme.colorScheme.onBackground,
										textDecoration = TextDecoration.Underline,
									),
								) {
									withLink(LinkAnnotation.Url(stringResource(R.string.contact_url))) {
										append(stringResource(R.string.contact_for_help))
									}
								}
								append(" ")
								append(stringResource(R.string.or_learn))
								append(" ")
								withStyle(
									style = SpanStyle(
										color = MaterialTheme.colorScheme.onBackground,
										textDecoration = TextDecoration.Underline,
									),
								) {
									withLink(LinkAnnotation.Url(stringResource(R.string.docs_url))) {
										append(stringResource(R.string.how_to_run_gateway))
									}
								}
							},
							textAlign = TextAlign.Center,
							style = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onBackground),
						)
					}
				}
			}

			items(
				uiState.items,
				key = { item ->
					when (item) {
						is ItemType.CountryItem -> item.locale.country
						is ItemType.GatewayItem -> item.gateway.identity
					}
				},
			) { item ->
				when (item) {
					is ItemType.CountryItem -> {
						CountryItem(
							query = uiState.query,
							countryItem = item,
							gatewayType = gatewayType,
							gatewayLocation = gatewayLocation,
							selectedKey = selectedKey,
							onSelectionChange = onSelect,
							onGatewayDetails = onNavigateToServerDetails,
							modifier = Modifier.padding(vertical = 4.dp),
							isQuicSettingsEnabled = canShowQuicLabel,
						)
					}

					is ItemType.GatewayItem -> {
						val gateway = item.gateway
						val locale = gateway.twoLetterCountryISO?.let { Locale("", it) }
						val showStreamDisplay = gatewayLocation == GatewayLocation.EXIT && gateway.asnKind == AsnKind.RESIDENTIAL

						SurfaceSelectionGroupButton(
							items = listOf(
								SelectionItem(
									onClick = { onSelect(gateway.identity) },
									leading = {
										val (icon, description) = gateway.getScoreIcon(gatewayType)
										Box(modifier = Modifier.padding(horizontal = 16.dp)) {
											Image(icon, contentDescription = description, modifier = Modifier.size(16.dp))
										}
									},
									trailing = {
										ServerDetailsTrailingContent(
											showStreamDisplay = showStreamDisplay,
											showQuicLabel = canShowQuicLabel && gateway.isQuicSupported(),
										) {
											onNavigateToServerDetails(gateway)
										}
									},
									title = {
										Text(
											gateway.name,
											style = MaterialTheme.typography.bodyLarge,
											color = MaterialTheme.colorScheme.onPrimaryContainer,
											maxLines = 1,
											overflow = TextOverflow.Ellipsis,
										)
									},
									description = {
										Text(
											text = gateway.serverLocation(locale?.displayCountry),
											maxLines = 1,
											overflow = TextOverflow.Ellipsis,
											style = MaterialTheme.typography.bodySmall,
											color = MaterialTheme.colorScheme.onBackground,
										)
									},
									selected = selectedKey == gateway.identity,
								),
							),
							shape = RectangleShape,
							background = MaterialTheme.colorScheme.primaryContainer,
							divider = false,
							anchorsPadding = 0.dp,
							modifier = Modifier,
						)
					}
				}
			}
		}
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Preview
@Composable
internal fun HopScreenPreview() {
	NymVPNTheme(Theme.default()) {
		Surface {
			HopScreenContent(
				uiState = HopUiState(),
				selectedKey = null,
				gatewayType = GatewayType.WG,
				canShowQuicLabel = false,
				gatewayLocation = GatewayLocation.ENTRY,
				initialGatewaysEmpty = true,
				isRefreshing = false,
				onRefresh = {},
				onQueryChange = {},
				onSelect = {},
				onNavigateToCensorship = {},
				onNavigateToServerDetails = {},
			)
		}
	}
}
