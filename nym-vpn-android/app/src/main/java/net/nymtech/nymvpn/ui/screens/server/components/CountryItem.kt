package net.nymtech.nymvpn.ui.screens.server.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.screens.server.GatewayLocation
import net.nymtech.nymvpn.ui.screens.server.ItemType
import net.nymtech.nymvpn.ui.screens.server.ServerListFilter
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.FavoriteIcon
import net.nymtech.nymvpn.util.extensions.getFlagImageVectorByName
import net.nymtech.nymvpn.util.extensions.getScoreIcon
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.GatewayType
import java.util.Locale

private val ListRowCornerRadius = 14.dp

private fun rowShape(roundTop: Boolean, roundBottom: Boolean): Shape = RoundedCornerShape(
	topStart = if (roundTop) ListRowCornerRadius else 0.dp,
	topEnd = if (roundTop) ListRowCornerRadius else 0.dp,
	bottomStart = if (roundBottom) ListRowCornerRadius else 0.dp,
	bottomEnd = if (roundBottom) ListRowCornerRadius else 0.dp,
)

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
	val rotationAngle by animateFloatAsState(targetValue = if (expanded) 90f else 0f)
	val ungrouped = if (countryItem.regions != null) countryItem.gateways.filter { it.region == null } else countryItem.gateways
	val regionsAreLastBlock = ungrouped.isEmpty()

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
			shape = rowShape(roundTop = true, roundBottom = !expanded),
			background = MaterialTheme.colorScheme.primaryContainer,
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
						isLastBlock = regionsAreLastBlock,
						onSelectionChange = onSelectionChange,
						onGatewayDetails = onGatewayDetails,
						onToggleFavorite = onToggleFavorite,
					)
				}
				if (ungrouped.isNotEmpty()) {
					GatewayCell(
						gatewayType = gatewayType,
						selectedKey = selectedKey,
						gateways = ungrouped,
						favoriteGatewayIds = favoriteGatewayIds,
						shape = rowShape(roundTop = false, roundBottom = true),
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
	shape: Shape,
	background: Color,
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
						horizontalArrangement = Arrangement.spacedBy(8.dp.scaledWidth()),
						verticalAlignment = Alignment.CenterVertically,
						modifier = Modifier
							.fillMaxHeight()
							.padding(end = 16.dp),
					) {
						FavoriteIcon(
							isFavorite = isFavorite,
							onToggleFavorite = onToggleFavorite,
							modifier = Modifier.size(iconSize),
						)
						IconButton(onClick = onDropDownClick) {
							Icon(
								Icons.AutoMirrored.Filled.KeyboardArrowRight,
								contentDescription = stringResource(if (expanded) R.string.collapse else R.string.expand),
								modifier = Modifier
									.graphicsLayer(rotationZ = rotationAngle)
									.size(iconSize),
							)
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
		shape = shape,
		background = background,
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
	isLastBlock: Boolean,
	onSelectionChange: (String) -> Unit,
	onGatewayDetails: (NymGateway) -> Unit,
	onToggleFavorite: (String, Boolean) -> Unit,
) {
	Column {
		gatewaysGroupByState.forEachIndexed { index, regionItem ->
			var isStateExpanded by rememberSaveable(key = "isStateExpanded_${gatewayLocation}_${filter}_${regionItem.region}") {
				mutableStateOf(regionItem.gateways.any { it.region.equals(selectedKey, true) || it.identity == selectedKey })
			}
			val stateRotationAngle by animateFloatAsState(targetValue = if (isStateExpanded) 90f else 0f, label = "StateItemRotation")
			val isLastRegion = isLastBlock && index == gatewaysGroupByState.lastIndex

			CountryDropDown(
				title = regionItem.region,
				countryCode = countryCode,
				rotationAngle = stateRotationAngle,
				expanded = isStateExpanded,
				isSelected = regionItem.region.equals(selectedKey, true),
				isFavorite = regionItem.isFavorite,
				gateways = regionItem.gateways,
				shape = rowShape(roundTop = false, roundBottom = isLastRegion && !isStateExpanded),
				background = MaterialTheme.colorScheme.surface.copy(alpha = 0.7f),
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
					favoriteGatewayIds = favoriteGatewayIds,
					shape = rowShape(roundTop = false, roundBottom = isLastRegion),
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
	selectedKey: String?,
	gateways: List<NymGateway>,
	favoriteGatewayIds: Set<String>,
	shape: Shape,
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
					val scoreIcon = gateway.getScoreIcon(gatewayType)
					if (scoreIcon != null) {
						val (icon, description) = scoreIcon
						Box(modifier = Modifier.padding(horizontal = 16.dp)) {
							Image(
								icon,
								contentDescription = description,
								modifier = Modifier.size(16.dp),
							)
						}
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
		shape = shape,
		background = MaterialTheme.colorScheme.surface.copy(alpha = 0.5f),
		divider = false,
		anchorsPadding = 0.dp,
	)
}
