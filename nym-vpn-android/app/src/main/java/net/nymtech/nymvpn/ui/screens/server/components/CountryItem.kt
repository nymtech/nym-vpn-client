package net.nymtech.nymvpn.ui.screens.server.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDropDown
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material.icons.rounded.StarBorder
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.VerticalDivider
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.screens.server.GatewayLocation
import net.nymtech.nymvpn.ui.screens.server.ItemType
import net.nymtech.nymvpn.ui.screens.server.ServerListFilter
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.getFlagImageVectorByName
import net.nymtech.nymvpn.util.extensions.getScoreIcon
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.GatewayType
import java.util.Locale

@Composable
fun CountryItem(
	query: String,
	countryItem: ItemType.CountryItem,
	gatewayType: GatewayType,
	gatewayLocation: GatewayLocation,
	filter: ServerListFilter,
	selectedKey: String?,
	favoriteGatewayIds: Set<String>,
	onSelectionChange: (String) -> Unit,
	onGatewayDetails: (NymGateway) -> Unit,
	onToggleFavorite: (String, Boolean) -> Unit,
	modifier: Modifier = Modifier,
) {
	val countryCode = remember(countryItem) { countryItem.locale.country.lowercase() }
	var expanded by rememberSaveable(key = "expanded_${gatewayLocation}_${filter}_${countryItem.locale.country}") {
		mutableStateOf(
			countryItem.gateways.any {
				it.identity == selectedKey ||
					it.region.equals(selectedKey, true) ||
					(
						countryCode == "us" &&
							query.takeIf { q -> q.isNotBlank() }?.let { q ->
								it.region?.contains(q, true)
							} ?: false
						)
			},
		)
	}
	val rotationAngle by animateFloatAsState(targetValue = if (expanded) 180f else 0f)

	Column(modifier = modifier) {
		CountryDropDown(
			title = countryItem.locale.displayCountry,
			countryCode = countryCode,
			country = countryItem.locale,
			rotationAngle = rotationAngle,
			expanded = expanded,
			gateways = countryItem.gateways,
			isSelected = countryCode == selectedKey,
			isFavorite = countryItem.isFavorite,
			onToggleFavorite = { onToggleFavorite(countryCode, countryItem.isFavorite) },
			onDropDownClick = {
				expanded = !expanded
			},
			onSelectionChange = {
				onSelectionChange(countryCode)
			},
		)
		AnimatedVisibility(
			visible = expanded,
			enter = expandVertically() + fadeIn(),
			exit = shrinkVertically() + fadeOut(),
		) {
			Column {
				if (!countryItem.regions.isNullOrEmpty()) {
					StateGroupedGatewayList(
						gatewaysGroupByState = countryItem.regions,
						countryCode = countryCode,
						selectedKey = selectedKey,
						country = countryItem.locale,
						gatewayType = gatewayType,
						gatewayLocation = gatewayLocation,
						filter = filter,
						favoriteGatewayIds = favoriteGatewayIds,
						onSelectionChange = onSelectionChange,
						onGatewayDetails = onGatewayDetails,
						onToggleFavorite = onToggleFavorite,
					)
				}
				val ungrouped = if (countryItem.regions != null) countryItem.gateways.filter { it.region == null } else countryItem.gateways
				if (ungrouped.isNotEmpty()) {
					GatewayCell(
						gatewayType = gatewayType,
						gatewayLocation = gatewayLocation,
						selectedKey = selectedKey,
						gateways = ungrouped,
						favoriteGatewayIds = favoriteGatewayIds,
						onSelectionChange = { onSelectionChange(it) },
						onGatewayDetails = { onGatewayDetails(it) },
						onToggleFavorite = onToggleFavorite,
					)
				}
			}
		}
	}
}

@Composable
private fun CountryDropDown(
	title: String,
	countryCode: String,
	rotationAngle: Float,
	expanded: Boolean,
	isSelected: Boolean,
	isFavorite: Boolean,
	country: Locale,
	gateways: List<NymGateway>,
	onDropDownClick: () -> Unit,
	onSelectionChange: () -> Unit,
	onToggleFavorite: () -> Unit,
) {
	val context = LocalContext.current
	SurfaceSelectionGroupButton(
		listOf(
			SelectionItem(
				onClick = { onSelectionChange() },
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
					Row(
						horizontalArrangement = Arrangement.spacedBy(16.dp),
						verticalAlignment = Alignment.CenterVertically,
						modifier = Modifier
							.fillMaxHeight()
							.padding(end = 16.dp),
					) {
						Icon(
							imageVector = if (isFavorite) Icons.Rounded.Star else Icons.Rounded.StarBorder,
							contentDescription = "Favorite",
							tint = if (isFavorite) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onBackground,
							modifier = Modifier
								.size(iconSize)
								.clickable { onToggleFavorite() },
						)
						Box(
							modifier = Modifier
								.clickable { onDropDownClick() }
								.fillMaxHeight(),
							contentAlignment = Alignment.Center,
						) {
							Row(
								horizontalArrangement = Arrangement.spacedBy(16.dp),
								verticalAlignment = Alignment.CenterVertically,
							) {
								VerticalDivider(modifier = Modifier.height(42.dp))
								Icon(
									Icons.Filled.ArrowDropDown,
									contentDescription = stringResource(if (expanded) R.string.collapse else R.string.expand),
									modifier = Modifier
										.graphicsLayer(rotationZ = rotationAngle)
										.size(iconSize),
								)
							}
						}
					}
				},
				title = {
					Text(
						text = title,
						style = MaterialTheme.typography.bodyLarge,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
				},
				description = {
					Text(
						"${gateways.size} ${stringResource(R.string.servers)}",
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
					)
				},
				selected = isSelected,
			),
		),
		shape = RectangleShape,
		background = MaterialTheme.colorScheme.surface,
		anchorsPadding = 0.dp,
	)
}

@Composable
private fun StateGroupedGatewayList(
	countryCode: String,
	selectedKey: String?,
	favoriteGatewayIds: Set<String>,
	country: Locale,
	gatewayType: GatewayType,
	gatewayLocation: GatewayLocation,
	filter: ServerListFilter,
	gatewaysGroupByState: List<ItemType.CountryItem.Region>,
	onSelectionChange: (String) -> Unit,
	onGatewayDetails: (NymGateway) -> Unit,
	onToggleFavorite: (String, Boolean) -> Unit,
) {
	Column {
		gatewaysGroupByState.forEach { regionItem ->
			var isStateExpanded by rememberSaveable(key = "isStateExpanded_${gatewayLocation}_${filter}_${regionItem.region}") {
				mutableStateOf(regionItem.gateways.any { it.region.equals(selectedKey, true) || it.identity == selectedKey })
			}
			val stateRotationAngle by animateFloatAsState(targetValue = if (isStateExpanded) 180f else 0f, label = "StateItemRotation")

			CountryDropDown(
				title = regionItem.region,
				countryCode = countryCode,
				rotationAngle = stateRotationAngle,
				expanded = isStateExpanded,
				isSelected = regionItem.region.equals(selectedKey, true),
				isFavorite = regionItem.isFavorite,
				gateways = regionItem.gateways,
				onDropDownClick = { isStateExpanded = !isStateExpanded },
				onSelectionChange = { onSelectionChange(regionItem.region) },
				onToggleFavorite = { onToggleFavorite(regionItem.region, regionItem.isFavorite) },
				country = country,
			)
			AnimatedVisibility(
				visible = isStateExpanded,
				enter = expandVertically() + fadeIn(),
				exit = shrinkVertically() + fadeOut(),
			) {
				GatewayCell(
					gateways = regionItem.gateways,
					selectedKey = selectedKey,
					gatewayType = gatewayType,
					gatewayLocation = gatewayLocation,
					favoriteGatewayIds = favoriteGatewayIds,
					onSelectionChange = onSelectionChange,
					onGatewayDetails = onGatewayDetails,
					onToggleFavorite = onToggleFavorite,
				)
			}
		}
	}
}

@Composable
private fun GatewayCell(
	gatewayType: GatewayType,
	gatewayLocation: GatewayLocation,
	selectedKey: String?,
	gateways: List<NymGateway>,
	favoriteGatewayIds: Set<String>,
	onSelectionChange: (String) -> Unit,
	onGatewayDetails: (NymGateway) -> Unit,
	onToggleFavorite: (String, Boolean) -> Unit,
) {
	SurfaceSelectionGroupButton(
		gateways.map { gateway ->
			val isFavorite = gateway.identity in favoriteGatewayIds
			SelectionItem(
				onClick = { onSelectionChange(gateway.identity) },
				leading = {
					val (icon, description) = gateway.getScoreIcon(gatewayType)
					Box(modifier = Modifier.padding(horizontal = 16.dp)) {
						Image(
							icon,
							contentDescription = description,
							modifier = Modifier.size(16.dp),
						)
					}
				},
				trailing = {
					ServerDetailsTrailingContent(
						isFavorite = isFavorite,
						onToggleFavorite = { onToggleFavorite(gateway.identity, isFavorite) },
						onInfoIconClick = { onGatewayDetails(gateway) },
					)
				},
				title = {
					Text(
						gateway.name,
						maxLines = 1,
						overflow = TextOverflow.Ellipsis,
						style = MaterialTheme.typography.bodyLarge,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
				},
				description = {
					Text(
						gateway.city ?: gateway.identity,
						maxLines = 1,
						overflow = TextOverflow.Ellipsis,
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
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
