package net.nymtech.nymvpn.ui.screens.server

import android.content.res.Configuration
import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.gestures.scrollBy
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.List
import androidx.compose.material.icons.rounded.AccessTime
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SecondaryTabRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Tab
import androidx.compose.material3.TabRowDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.snapshotFlow
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
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
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarEvent
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.screens.server.components.CountryHeaderRow
import net.nymtech.nymvpn.ui.screens.server.components.GatewayRow
import net.nymtech.nymvpn.ui.screens.server.components.RegionHeaderRow
import net.nymtech.nymvpn.ui.screens.server.components.ExitServerDetailsModal
import net.nymtech.nymvpn.ui.screens.server.components.QuicInfoMessage
import net.nymtech.nymvpn.ui.screens.server.components.ServerDetailsModalBody
import net.nymtech.nymvpn.ui.screens.server.components.ServerDetailsTrailingContent
import net.nymtech.nymvpn.ui.screens.server.components.showStreamDisplay
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.getScoreIcon
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.safePopBackStack
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.GatewayType
import java.util.Locale

@Composable
fun ServerScreen(
	gatewayLocation: GatewayLocation,
	appUiState: AppUiState,
	navBarEvent: NavBarEvent?,
	onNavBarEventConsume: () -> Unit,
	onLocationChange: (GatewayLocation) -> Unit = {},
	viewModel: ServerViewModel = hiltViewModel(),
) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val navController = LocalNavController.current
	val context = LocalContext.current
	val coroutineScope = rememberCoroutineScope()
	val locationSupportLink = stringResource(R.string.location_support_link)

	var refreshing by remember { mutableStateOf(false) }
	var selectedLocation by remember { mutableStateOf(gatewayLocation) }

	var showLocationTooltip by remember { mutableStateOf(false) }
	var showExitServerTooltip by remember { mutableStateOf(false) }

	LaunchedEffect(selectedLocation) {
		onLocationChange(selectedLocation)
	}

	LaunchedEffect(navBarEvent, selectedLocation) {
		when (navBarEvent) {
			NavBarEvent.EntryLocationInfoClicked, NavBarEvent.ExitLocationInfoClicked -> {
				when (selectedLocation) {
					GatewayLocation.ENTRY -> showLocationTooltip = true
					GatewayLocation.EXIT -> showExitServerTooltip = true
				}
				onNavBarEventConsume()
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

	val gatewayType = remember(selectedLocation) {
		when (appUiState.vpnConfig.mode) {
			Tunnel.Mode.FIVE_HOP_MIXNET -> {
				when (selectedLocation) {
					GatewayLocation.EXIT -> GatewayType.MIXNET_EXIT
					GatewayLocation.ENTRY -> GatewayType.MIXNET_ENTRY
				}
			}
			Tunnel.Mode.TWO_HOP_MIXNET -> GatewayType.WG
		}
	}

	val selectedKey = remember(selectedLocation) {
		when (selectedLocation) {
			GatewayLocation.ENTRY -> appUiState.entryPointId
			GatewayLocation.EXIT -> appUiState.exitPointId
		}
	}

	val canShowQuicLabel = remember(selectedLocation) {
		selectedLocation == GatewayLocation.ENTRY &&
			appUiState.vpnConfig.mode == Tunnel.Mode.TWO_HOP_MIXNET &&
			appUiState.settings.quicEnabled
	}

	val isRandomSelected = remember(selectedLocation, appUiState.vpnConfig.entryPoint, appUiState.vpnConfig.exitPoint) {
		when (selectedLocation) {
			GatewayLocation.ENTRY -> appUiState.vpnConfig.entryPoint is EntryPoint.Random
			GatewayLocation.EXIT -> appUiState.vpnConfig.exitPoint is ExitPoint.Random
		}
	}
	val isSafestSelected = remember(selectedLocation, appUiState.vpnConfig.entryPoint, appUiState.vpnConfig.exitPoint) {
		when (selectedLocation) {
			GatewayLocation.ENTRY -> appUiState.vpnConfig.entryPoint is EntryPoint.Auto
			GatewayLocation.EXIT -> appUiState.vpnConfig.exitPoint is ExitPoint.Auto
		}
	}

	LaunchedEffect(selectedLocation, gatewayType) {
		viewModel.load(
			type = gatewayType,
			isExitScreen = selectedLocation == GatewayLocation.EXIT,
			selectedKey = selectedKey,
			tunnelMode = appUiState.vpnConfig.mode,
			isQuicEnabled = appUiState.settings.quicEnabled,
		)
	}

	LaunchedEffect(refreshing) {
		if (refreshing) viewModel.onRefresh()
		refreshing = false
	}

	ServerScreenContent(
		uiState = uiState,
		selectedKey = selectedKey,
		gatewayType = gatewayType,
		canShowQuicLabel = canShowQuicLabel,
		isRandomSelected = isRandomSelected,
		isSafestSelected = isSafestSelected,
		gatewayLocation = selectedLocation,
		isRefreshing = refreshing,
		onRefresh = { refreshing = true },
		onQueryChange = { viewModel.onQueryChange(it) },
		onSelect = { id ->
			coroutineScope.launch {
				viewModel.onSelected(id, selectedLocation)
				navController.safePopBackStack()
			}
		},
		onLocationSelect = { location -> selectedLocation = location },
		onFilterSelect = { viewModel.onFilterSelected(it) },
		onToggleFavorite = { id, isFavorite -> viewModel.onToggleFavorite(id, isFavorite) },
		onToggleExpanded = { viewModel.onToggleExpanded(it) },
		onNavigateToCensorship = { navController.navigate(Route.Censorship) },
		onNavigateToServerDetails = { gateway ->
			navController.goFromRoot(Route.ServerDetails(gateway.identity, selectedLocation.name))
		},
	)
}

@Composable
internal fun ServerScreenContent(
	uiState: ServerUiState,
	selectedKey: String?,
	gatewayType: GatewayType,
	canShowQuicLabel: Boolean,
	isRandomSelected: Boolean,
	isSafestSelected: Boolean,
	gatewayLocation: GatewayLocation,
	isRefreshing: Boolean,
	onRefresh: () -> Unit,
	onQueryChange: (String) -> Unit,
	onSelect: (String) -> Unit,
	onLocationSelect: (GatewayLocation) -> Unit,
	onFilterSelect: (ServerListFilter) -> Unit,
	onToggleFavorite: (String, Boolean) -> Unit,
	onToggleExpanded: (ServerRow.Expandable) -> Unit,
	onNavigateToCensorship: () -> Unit,
	onNavigateToServerDetails: (NymGateway) -> Unit,
) {
	val pullRefreshState = rememberPullToRefreshState()
	val listState = rememberLazyListState()
	var hasScrolled by remember { mutableStateOf(false) }

	val scrollTargetKey = remember(uiState.rows, selectedKey) {
		uiState.rows.firstOrNull { it.represents(selectedKey) }?.key
	}

	LaunchedEffect(scrollTargetKey) {
		if (hasScrolled || scrollTargetKey == null) return@LaunchedEffect
		val index = HEADER_ITEM_COUNT + uiState.rows.indexOfFirst { it.key == scrollTargetKey }
		hasScrolled = true
		val viewportHeight = snapshotFlow { listState.layoutInfo.viewportSize.height }.first { it > 0 }
		listState.scrollToItem(index)
		val target = listState.layoutInfo.visibleItemsInfo.firstOrNull { it.key == scrollTargetKey } ?: return@LaunchedEffect
		val desiredOffset = (viewportHeight * SELECTED_ITEM_VIEWPORT_FRACTION).toInt()
		listState.scrollBy((target.offset - desiredOffset).toFloat())
	}

	PullToRefreshBox(
		state = pullRefreshState,
		isRefreshing = isRefreshing,
		onRefresh = onRefresh,
		modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
	) {
		LazyColumn(
			state = listState,
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.Top,
			modifier = Modifier
				.fillMaxSize()
				.windowInsetsPadding(WindowInsets.navigationBars)
				.imePadding()
				.padding(horizontal = 16.dp.scaledWidth()),
		) {
			item {
				GatewayLocationTabs(
					selected = gatewayLocation,
					onSelect = onLocationSelect,
					modifier = Modifier
						.fillMaxWidth()
						.padding(top = 8.dp.scaledHeight()),
				)
			}

			item {
				Column(
					verticalArrangement = Arrangement.spacedBy(14.dp.scaledHeight()),
					modifier = Modifier
						.padding(top = 12.dp.scaledHeight()),
				) {
					if (canShowQuicLabel) {
						QuicInfoMessage(onNavigateToQuicSettings = onNavigateToCensorship)
					}
					CustomTextField(
						value = uiState.query,
						onValueChange = onQueryChange,
						modifier = Modifier
							.fillMaxWidth()
							.height(48.dp.scaledHeight().coerceAtLeast(48.dp))
							.background(MaterialTheme.colorScheme.background, RoundedCornerShape(12.dp)),
						placeholder = {
							Text(
								stringResource(R.string.server_search_text),
								color = MaterialTheme.colorScheme.onBackground,
							)
						},
						singleLine = true,
						leading = {
							Icon(
								Icons.Rounded.Search,
								contentDescription = stringResource(R.string.search),
								modifier = Modifier
									.size(iconSize),
							)
						},
						label = {},
						showClearIcon = true,
						textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onBackground),
						containerColor = MaterialTheme.colorScheme.background,
					)
					Text(
						text = stringResource(
							R.string.server_countries_nodes_combined_text,
							pluralStringResource(R.plurals.server_countries_count_text, uiState.countryCount, uiState.countryCount),
							pluralStringResource(R.plurals.server_nodes_count_text, uiState.nodeCount, uiState.nodeCount),
						),
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
					)
					if (uiState.error && uiState.hasLocations) {
						Text(
							text = stringResource(R.string.server_list_update_failed),
							style = MaterialTheme.typography.bodySmall,
							color = MaterialTheme.colorScheme.error,
						)
					}
					ServerFilterPills(
						selected = uiState.filter,
						onSelect = onFilterSelect,
						modifier = Modifier.fillMaxWidth(),
					)
				}
			}
			item {
				VerticalDivider(Modifier.height(12.dp))
			}
			items(uiState.rows, key = { it.key }, contentType = { it::class }) { row ->
				val rowModifier = Modifier.animateItem()
				when (row) {
					ServerRow.Safest -> AutoOptionRow(
						icon = R.drawable.ic_safest,
						title = R.string.gateway_safest,
						selected = isSafestSelected,
						onClick = { onSelect("Auto") },
						modifier = rowModifier.padding(bottom = 8.dp),
					)

					ServerRow.Random -> AutoOptionRow(
						icon = R.drawable.ic_random,
						title = R.string.gateway_random,
						selected = isRandomSelected,
						onClick = { onSelect("Random") },
						modifier = rowModifier.padding(bottom = 4.dp),
					)

					is ServerRow.CountryHeader -> CountryHeaderRow(
						row = row,
						isSelected = row.represents(selectedKey),
						onSelectionChange = onSelect,
						onToggleExpanded = onToggleExpanded,
						onToggleFavorite = onToggleFavorite,
						modifier = rowModifier,
					)

					is ServerRow.RegionHeader -> RegionHeaderRow(
						row = row,
						isSelected = row.represents(selectedKey),
						onSelectionChange = onSelect,
						onToggleExpanded = onToggleExpanded,
						onToggleFavorite = onToggleFavorite,
						modifier = rowModifier,
					)

					is ServerRow.Gateway -> GatewayRow(
						row = row,
						gatewayType = gatewayType,
						gatewayLocation = gatewayLocation,
						isSelected = row.represents(selectedKey),
						onSelectionChange = onSelect,
						onGatewayDetails = onNavigateToServerDetails,
						onToggleFavorite = onToggleFavorite,
						modifier = rowModifier,
					)

					is ServerRow.SearchResult -> SearchResultRow(
						row = row,
						gatewayType = gatewayType,
						gatewayLocation = gatewayLocation,
						isSelected = row.represents(selectedKey),
						onSelect = onSelect,
						onToggleFavorite = onToggleFavorite,
						onNavigateToServerDetails = onNavigateToServerDetails,
						modifier = rowModifier,
					)
				}
			}

			if (!uiState.hasLocations && uiState.isEmpty) {
				item {
					Box(
						modifier = Modifier
							.fillMaxWidth()
							.padding(top = 24.dp.scaledHeight())
							.padding(horizontal = 16.dp.scaledWidth()),
						contentAlignment = Alignment.Center,
					) {
						val emptyStateText = when {
							uiState.error -> stringResource(R.string.country_load_failure)
							uiState.filter == ServerListFilter.FAVORITES -> stringResource(R.string.server_no_favorites_text)
							uiState.filter == ServerListFilter.RECENT && !uiState.isLoading -> stringResource(R.string.server_no_recents_text)
							else -> stringResource(R.string.loading)
						}
						Text(
							emptyStateText,
							style = MaterialTheme.typography.bodyMedium.copy(
								color = if (uiState.error) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onBackground,
							),
							textAlign = TextAlign.Center,
						)
					}
				}
			}

			if (uiState.query.isNotBlank() && !uiState.hasLocations && !uiState.error) {
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
		}
		if (uiState.isSyncing && !isRefreshing) {
			LinearProgressIndicator(
				modifier = Modifier
					.align(Alignment.TopCenter)
					.fillMaxWidth()
					.height(2.dp),
			)
		}
	}
}

private const val SELECTED_ITEM_VIEWPORT_FRACTION = 1f / 3

private const val HEADER_ITEM_COUNT = 3

@Composable
private fun SearchResultRow(
	row: ServerRow.SearchResult,
	gatewayType: GatewayType,
	gatewayLocation: GatewayLocation,
	isSelected: Boolean,
	onSelect: (String) -> Unit,
	onToggleFavorite: (String, Boolean) -> Unit,
	onNavigateToServerDetails: (NymGateway) -> Unit,
	modifier: Modifier = Modifier,
) {
	val gateway = row.gateway
	val locale = gateway.twoLetterCountryISO?.let { Locale("", it) }

	SurfaceSelectionGroupButton(
		items = listOf(
			SelectionItem(
				onClick = { onSelect(gateway.identity) },
				leading = {
					val scoreIcon = gateway.getScoreIcon(gatewayType)
					if (scoreIcon != null) {
						val (icon, description) = scoreIcon
						Box(modifier = Modifier.padding(horizontal = 16.dp)) {
							Image(icon, contentDescription = description, modifier = Modifier.size(16.dp))
						}
					}
				},
				trailing = {
					ServerDetailsTrailingContent(
						showStreamDisplay = gateway.showStreamDisplay(gatewayLocation),
						isFavorite = row.isFavorite,
						onToggleFavorite = { onToggleFavorite(gateway.identity, row.isFavorite) },
						onInfoIconClick = { onNavigateToServerDetails(gateway) },
					)
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
				selected = isSelected,
			),
		),
		shape = RoundedCornerShape(14.dp),
		background = MaterialTheme.colorScheme.surface.copy(alpha = 0.5f),
		divider = false,
		anchorsPadding = 0.dp,
		modifier = modifier.padding(vertical = 4.dp),
	)
}

@Composable
private fun AutoOptionRow(@DrawableRes icon: Int, @StringRes title: Int, selected: Boolean, onClick: () -> Unit, modifier: Modifier = Modifier) {
	SurfaceSelectionGroupButton(
		items = listOf(
			SelectionItem(
				onClick = onClick,
				leading = {
					Box(modifier = Modifier.padding(horizontal = 16.dp)) {
						Icon(
							imageVector = ImageVector.vectorResource(icon),
							contentDescription = null,
							modifier = Modifier.size(iconSize),
						)
					}
				},
				title = { Text(stringResource(title), style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onPrimaryContainer) },
				selected = selected,
			),
		),
		shape = RoundedCornerShape(14.dp),
		background = MaterialTheme.colorScheme.primaryContainer,
		anchorsPadding = 0.dp,
		modifier = modifier,
	)
}

@Composable
private fun GatewayLocationTabs(selected: GatewayLocation, onSelect: (GatewayLocation) -> Unit, modifier: Modifier = Modifier) {
	val selectedTabIndex = if (selected == GatewayLocation.ENTRY) 0 else 1
	val tabs = listOf(GatewayLocation.ENTRY to stringResource(R.string.server_entry_tab), GatewayLocation.EXIT to stringResource(R.string.server_exit_tab))

	SecondaryTabRow(
		selectedTabIndex = selectedTabIndex,
		modifier = modifier,
		containerColor = MaterialTheme.colorScheme.background,
		contentColor = MaterialTheme.colorScheme.background,
		indicator = {
			TabRowDefaults.SecondaryIndicator(
				modifier = Modifier.tabIndicatorOffset(selectedTabIndex = selectedTabIndex, matchContentSize = false),
				color = MaterialTheme.colorScheme.primary,
			)
		},
	) {
		tabs.forEachIndexed { index, (location, title) ->
			val tabSelected = index == selectedTabIndex
			Tab(
				selected = tabSelected,
				onClick = { onSelect(location) },
			) {
				Text(
					text = title,
					style = MaterialTheme.typography.bodyLarge,
					color = if (tabSelected) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onBackground,
					modifier = Modifier.padding(top = 10.dp, bottom = 14.dp),
				)
			}
		}
	}
}

@Composable
private fun ServerFilterPills(selected: ServerListFilter, onSelect: (ServerListFilter) -> Unit, modifier: Modifier = Modifier) {
	val filters = listOf(
		Triple(ServerListFilter.FAVORITES, Icons.Rounded.Star, stringResource(R.string.server_favorites_tab)),
		Triple(ServerListFilter.RECENT, Icons.Rounded.AccessTime, stringResource(R.string.server_recent_tab)),
		Triple(ServerListFilter.ALL_SERVERS, Icons.AutoMirrored.Rounded.List, stringResource(R.string.server_all_tab)),
	)
	Row(
		horizontalArrangement = Arrangement.spacedBy(8.dp.scaledWidth(), Alignment.CenterHorizontally),
		modifier = modifier,
	) {
		filters.forEach { (filter, icon, label) ->
			val isSelected = filter == selected
			val contentColor = if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onPrimaryContainer
			val pillModifier = if (isSelected) {
				Modifier
					.clip(RoundedCornerShape(50))
					.border(width = 1.dp, color = MaterialTheme.colorScheme.primary, shape = RoundedCornerShape(50))
			} else {
				Modifier
			}
			Row(
				verticalAlignment = Alignment.CenterVertically,
				horizontalArrangement = Arrangement.spacedBy(4.dp.scaledWidth()),
				modifier = pillModifier
					.clickable { onSelect(filter) }
					.padding(horizontal = 12.dp.scaledWidth(), vertical = 10.dp.scaledHeight()),
			) {
				Icon(
					imageVector = icon,
					contentDescription = null,
					tint = contentColor,
					modifier = Modifier.size(24.dp),
				)
				Text(
					text = label,
					style = MaterialTheme.typography.bodyMedium,
					color = contentColor,
				)
			}
		}
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Preview
@Composable
internal fun ServerScreenPreview() {
	NymVPNTheme(Theme.default()) {
		Surface {
			ServerScreenContent(
				uiState = ServerUiState(),
				selectedKey = null,
				gatewayType = GatewayType.WG,
				canShowQuicLabel = false,
				isRandomSelected = false,
				isSafestSelected = true,
				gatewayLocation = GatewayLocation.EXIT,
				isRefreshing = false,
				onRefresh = {},
				onQueryChange = {},
				onSelect = {},
				onLocationSelect = {},
				onFilterSelect = {},
				onToggleFavorite = { _, _ -> },
				onToggleExpanded = {},
				onNavigateToCensorship = {},
				onNavigateToServerDetails = {},
			)
		}
	}
}
