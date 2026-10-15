package net.nymtech.nymvpn.ui.screens.server.components

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.FlagResources
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.screens.server.ServerRow
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.FavoriteIcon
import net.nymtech.nymvpn.util.extensions.getScoreIcon
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.GatewayType

private val ListRowCornerRadius = 14.dp

private fun rowShape(roundTop: Boolean, roundBottom: Boolean): Shape = RoundedCornerShape(
	topStart = if (roundTop) ListRowCornerRadius else 0.dp,
	topEnd = if (roundTop) ListRowCornerRadius else 0.dp,
	bottomStart = if (roundBottom) ListRowCornerRadius else 0.dp,
	bottomEnd = if (roundBottom) ListRowCornerRadius else 0.dp,
)

@Composable
fun CountryHeaderRow(row: ServerRow.Country, onSelect: (String) -> Unit, onToggleExpanded: (String, Boolean) -> Unit, onToggleFavorite: (String, Boolean) -> Unit, modifier: Modifier = Modifier) {
	GroupHeader(
		title = row.displayName,
		countryCode = row.code,
		flagDescription = stringResource(R.string.country_flag, row.displayName),
		nodeCount = row.nodeCount,
		expanded = row.expanded,
		isSelected = row.isSelected,
		isFavorite = row.isFavorite,
		shape = rowShape(roundTop = true, roundBottom = !row.expanded),
		background = MaterialTheme.colorScheme.primaryContainer,
		onSelect = { onSelect(row.code) },
		onToggleExpanded = { onToggleExpanded(row.key, row.expanded) },
		onToggleFavorite = { onToggleFavorite(row.code, row.isFavorite) },
		modifier = modifier,
	)
}

@Composable
fun RegionHeaderRow(row: ServerRow.Region, onSelect: (String) -> Unit, onToggleExpanded: (String, Boolean) -> Unit, onToggleFavorite: (String, Boolean) -> Unit, modifier: Modifier = Modifier) {
	GroupHeader(
		title = row.name,
		countryCode = row.countryCode,
		flagDescription = stringResource(R.string.country_flag, row.name),
		nodeCount = row.nodeCount,
		expanded = row.expanded,
		isSelected = row.isSelected,
		isFavorite = row.isFavorite,
		shape = rowShape(roundTop = false, roundBottom = row.roundBottom),
		background = MaterialTheme.colorScheme.surface.copy(alpha = 0.7f),
		onSelect = { onSelect(row.name) },
		onToggleExpanded = { onToggleExpanded(row.key, row.expanded) },
		onToggleFavorite = { onToggleFavorite(row.name, row.isFavorite) },
		modifier = modifier,
	)
}

@Composable
private fun GroupHeader(
	title: String,
	countryCode: String,
	flagDescription: String,
	nodeCount: Int,
	expanded: Boolean,
	isSelected: Boolean,
	isFavorite: Boolean,
	shape: Shape,
	background: Color,
	onSelect: () -> Unit,
	onToggleExpanded: () -> Unit,
	onToggleFavorite: () -> Unit,
	modifier: Modifier = Modifier,
) {
	val rotationAngle by animateFloatAsState(targetValue = if (expanded) 90f else 0f, label = "GroupHeaderChevron")
	SurfaceSelectionGroupButton(
		listOf(
			SelectionItem(
				onClick = onSelect,
				leading = {
					Box(modifier = Modifier.padding(horizontal = 16.dp)) {
						Image(
							ImageVector.vectorResource(FlagResources.forCountry(countryCode)),
							contentDescription = flagDescription,
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
						IconButton(onClick = onToggleExpanded) {
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
						"$nodeCount ${stringResource(R.string.servers)}",
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
		modifier = modifier,
	)
}

@Composable
fun GatewayRow(
	row: ServerRow.Gateway,
	gatewayType: GatewayType,
	onSelect: (String) -> Unit,
	onGatewayDetails: (NymGateway) -> Unit,
	onToggleFavorite: (String, Boolean) -> Unit,
	modifier: Modifier = Modifier,
) {
	val gateway = row.gateway
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
						isFavorite = row.isFavorite,
						onToggleFavorite = { onToggleFavorite(gateway.identity, row.isFavorite) },
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
						row.subtitle,
						maxLines = 1,
						overflow = TextOverflow.Ellipsis,
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
					)
				},
				selected = row.isSelected,
			),
		),
		shape = rowShape(roundTop = row.standalone, roundBottom = row.roundBottom),
		background = MaterialTheme.colorScheme.surface.copy(alpha = 0.5f),
		divider = false,
		anchorsPadding = 0.dp,
		modifier = modifier,
	)
}
