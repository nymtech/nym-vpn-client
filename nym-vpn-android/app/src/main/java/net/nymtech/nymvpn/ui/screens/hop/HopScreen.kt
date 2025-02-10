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
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
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
import androidx.compose.runtime.derivedStateOf
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
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextLinkStyles
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withLink
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
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
import net.nymtech.nymvpn.util.extensions.scoreSorted
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib.GatewayType
import nym_vpn_lib.Score
import java.text.Collator
import java.util.Locale

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HopScreen(gatewayLocation: GatewayLocation, appViewModel: AppViewModel, appUiState: AppUiState, viewModel: HopViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val context = LocalContext.current
	val navController = LocalNavController.current

	var refreshing by remember { mutableStateOf(false) }
	var query by rememberSaveable { mutableStateOf("") }
	val pullRefreshState = rememberPullToRefreshState()

	val currentLocale = ConfigurationCompat.getLocales(context.resources.configuration)[0]
	val collator = Collator.getInstance(currentLocale)

	val selectedKey = remember {
		when (gatewayLocation) {
			GatewayLocation.ENTRY -> appUiState.entryPointId
			GatewayLocation.EXIT -> appUiState.exitPointId
		}
	}

	var showLocationTooltip by remember { mutableStateOf(false) }

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = {
					NavTitle(
						when (gatewayLocation) {
							GatewayLocation.EXIT -> stringResource(R.string.exit)
							GatewayLocation.ENTRY -> stringResource(R.string.entry)
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

	val gateways = remember(appUiState.gateways) {
		when (gatewayType) {
			GatewayType.MIXNET_ENTRY -> appUiState.gateways.entryGateways
			GatewayType.MIXNET_EXIT -> appUiState.gateways.exitGateways
			GatewayType.WG -> appUiState.gateways.wgGateways
		}
	}

	val countries = remember(uiState.query) {
		derivedStateOf {
			val query = uiState.query.lowercase()
			gateways.distinctBy { it.twoLetterCountryISO }.filter { it.twoLetterCountryISO != null }
				.map {
					Locale(it.twoLetterCountryISO!!, it.twoLetterCountryISO!!)
				}.filter {
					it.displayCountry.lowercase().contains(query) || it.country.lowercase().contains(query) || it.isO3Country.lowercase().contains(query)
				}
				.sortedWith(compareBy(collator) { it.displayCountry })
		}
	}.value

	val queriedGateways = remember(uiState.query) {
		derivedStateOf {
			if (!uiState.query.isBlank()) {
				gateways.filter { it.identity.lowercase().contains(uiState.query) }.sortedWith(
					compareBy(collator) { it.identity },
				)
			} else {
				emptyList()
			}
		}
	}.value

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

	@Composable
	fun getScoreIcon(gateway: NymGateway): ImageVector {
		val score = when (gatewayType) {
			GatewayType.MIXNET_ENTRY, GatewayType.MIXNET_EXIT -> gateway.mixnetScore
			GatewayType.WG -> gateway.wgScore
		}
		return when (score) {
			Score.HIGH -> ImageVector.vectorResource(R.drawable.bars_3)
			Score.MEDIUM -> ImageVector.vectorResource(R.drawable.bars_2)
			Score.LOW -> ImageVector.vectorResource(R.drawable.bar_1)
			Score.NONE -> ImageVector.vectorResource(R.drawable.faq)
			null -> ImageVector.vectorResource(R.drawable.faq)
		}
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
				.fillMaxSize().windowInsetsPadding(WindowInsets.navigationBars).imePadding(),
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
					CustomTextField(
						value = query,
						onValueChange = {
							query = it
							viewModel.onQueryChange(it)
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
			if (query != "" && countries.isEmpty() && queriedGateways.isEmpty() && gateways.isNotEmpty()) {
				item {
					val annotatedString = buildAnnotatedString {
						append(stringResource(R.string.try_another_server_name))
						append(" ")
						withLink(
							link = LinkAnnotation.Url(
								url = stringResource(R.string.contact_url),
								styles = TextLinkStyles(
									style = SpanStyle(
										textDecoration = TextDecoration.Underline,
									),

								),
							),
						) {
							append(stringResource(R.string.contact_for_help))
						}
						append(" ")
						append(stringResource(R.string.or_learn))
						append(" ")
						withLink(
							link = LinkAnnotation.Url(
								url = stringResource(R.string.docs_url),
								styles = TextLinkStyles(
									style = SpanStyle(
										textDecoration = TextDecoration.Underline,
									),
								),
							),
						) {
							append(stringResource(R.string.how_to_run_gateway))
						}
					}
					Column(
						horizontalAlignment = Alignment.CenterHorizontally,
						verticalArrangement = Arrangement.spacedBy(5.dp.scaledHeight(), Alignment.Top),
						modifier = Modifier.padding(horizontal = 16.dp.scaledWidth()).fillMaxWidth(),
					) {
						Text(
							stringResource(R.string.no_results_found),
							textAlign = TextAlign.Center,
							style = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onBackground).copy(
								letterSpacing = 0.5.sp,
								fontWeight = FontWeight(400),
							),
						)
						Text(
							annotatedString,
							textAlign = TextAlign.Center,
							style = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.outline).copy(
								letterSpacing = 0.5.sp,
								fontWeight = FontWeight(400),
							),
						)
					}
				}
			}
			items(countries, key = { it.displayCountry }) { country ->
				Column(modifier = Modifier.padding(bottom = 8.dp)) {
					var expanded by remember { mutableStateOf(false) }

					LaunchedEffect(Unit) {
						expanded = gateways.filter { it.twoLetterCountryISO == country.country.lowercase() }.any { it.identity == selectedKey }
					}

					val rotationAngle by animateFloatAsState(targetValue = if (expanded) 180f else 0f)
					val countryCode = country.country.lowercase()
					SurfaceSelectionGroupButton(
						listOf(
							SelectionItem(
								onClick = {
									onSelectionChange(countryCode)
								},
								leading = {
									val icon = ImageVector.vectorResource(
										context.getFlagImageVectorByName(countryCode),
									)
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
								title = { Text(country.displayCountry, style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
								description = {
									Text(
										"${gateways.count { it.twoLetterCountryISO == countryCode }}  ${stringResource(R.string.servers)}",
										style = MaterialTheme.typography.bodySmall.copy(MaterialTheme.colorScheme.outline),
									)
								},
								selected = countryCode == selectedKey,
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
							gateways.filter { it.twoLetterCountryISO == countryCode }
								.scoreSorted(appUiState.settings.vpnMode).map { gateway ->
									SelectionItem(
										onClick = {
											onSelectionChange(gateway.identity)
										},
										leading = {
											val icon = getScoreIcon(gateway)
											Image(
												icon,
												icon.name,
												modifier = Modifier.height(16.dp).width(15.dp),
											)
										},
										// TODO disable info dialog for now
// 									trailing = {
// 										Row(
// 											horizontalArrangement = Arrangement.spacedBy(16.dp),
// 											verticalAlignment = Alignment.CenterVertically,
// 										) {
// 											val icon = Icons.Outlined.Info
// 											VerticalDivider(modifier = Modifier.height(42.dp))
// 											Icon(icon, icon.name, Modifier.size(iconSize))
// 										}
// 									},
										title = {
											Text(
												gateway.name,
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
										selected = selectedKey == gateway.identity,
									)
								},
							shape = RectangleShape,
							background = MaterialTheme.colorScheme.background,
							divider = false,
						)
					}
					if (expanded && queriedGateways.isNotEmpty() &&
						countries.lastOrNull() == country
					) {
						Spacer(modifier = Modifier.height(24.dp.scaledHeight()))
					}
				}
			}
			if (queriedGateways.isNotEmpty()) {
				item {
					SurfaceSelectionGroupButton(
						queriedGateways.scoreSorted(appUiState.settings.vpnMode).map { gateway ->
							val locale = gateway.twoLetterCountryISO?.let {
								Locale(it, it)
							}
							SelectionItem(
								onClick = {
									onSelectionChange(gateway.identity)
								},
								leading = {
									val icon = getScoreIcon(gateway)
									Image(
										icon,
										icon.name,
										modifier =
										Modifier.height(16.dp).width(15.dp),
									)
								},
								// TODO disable for now
// 								trailing = {
// 									Row(
// 										horizontalArrangement = Arrangement.spacedBy(16.dp),
// 										verticalAlignment = Alignment.CenterVertically,
// 									) {
// 										val icon = Icons.Outlined.Info
// 										VerticalDivider(modifier = Modifier.height(42.dp))
// 										Icon(icon, icon.name, Modifier.size(iconSize))
// 									}
// 								},
								title = {
									Text(
										gateway.name,
										maxLines = 1,
										overflow = TextOverflow.Ellipsis,
										style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
									)
								},
								description = {
									Text(
										"${locale?.displayCountry ?: stringResource(R.string.unknown)}, ${gateway.identity}",
										maxLines = 1,
										overflow = TextOverflow.Ellipsis,
										style = MaterialTheme.typography.bodySmall.copy(MaterialTheme.colorScheme.outline),
									)
								},
								selected = selectedKey == gateway.identity,
							)
						},
						shape = RectangleShape,
						background = MaterialTheme.colorScheme.background,
						divider = false,
					)
				}
			}
		}
	}
}
