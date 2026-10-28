// Composables for individual rows of the server list (country, region and gateway).
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.screens.server.GatewayLocation
import net.nymtech.nymvpn.ui.screens.server.ServerRow
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.FavoriteIcon
import net.nymtech.nymvpn.util.extensions.getFlagImageVectorByName
import net.nymtech.nymvpn.util.extensions.getScoreIcon
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.AsnKind
import nym_vpn_lib_types.GatewayType
import java.util.Locale

private val ListRowCornerRadius = 14.dp
private val GroupSpacing = 4.dp

private fun rowShape(roundTop: Boolean, roundBottom: Boolean): Shape = RoundedCornerShape(
	topStart = if (roundTop) ListRowCornerRadius else 0.dp,
	topEnd = if (roundTop) ListRowCornerRadius else 0.dp,
	bottomStart = if (roundBottom) ListRowCornerRadius else 0.dp,
	bottomEnd = if (roundBottom) ListRowCornerRadius else 0.dp,
)

@Composable
internal fun CountryHeaderRow(
	row: ServerRow.CountryHeader,
	isSelected: Boolean,
	onSelectionChange: (String) -> Unit,
	onToggleExpanded: (ServerRow.Expandable) -> Unit,
	onToggleFavorite: (String, Boolean) -> Unit,
	modifier: Modifier = Modifier,
) {
	val rotationAngle by animateFloatAsState(targetValue = if (row.isExpanded) 90f else 0f)
	CountryDropDown(
		title = row.locale.displayCountry,
		countryCode = row.countryCode,
		country = row.locale,
		rotationAngle = rotationAngle,
		expanded = row.isExpanded,
		gatewayCount = row.gatewayCount,
		isSelected = isSelected,
		isFavorite = row.isFavorite,
		shape = rowShape(roundTop = true, roundBottom = !row.isExpanded),
		background = MaterialTheme.colorScheme.primaryContainer,
		onToggleFavorite = { onToggleFavorite(row.countryCode, row.isFavorite) },
		onDropDownClick = { onToggleExpanded(row) },
		onSelectionChange = { onSelectionChange(row.countryCode) },
		modifier = modifier.padding(top = GroupSpacing, bottom = if (row.isExpanded) 0.dp else GroupSpacing),
	)
}

@Composable
internal fun RegionHeaderRow(
	row: ServerRow.RegionHeader,
	isSelected: Boolean,
	onSelectionChange: (String) -> Unit,
	onToggleExpanded: (ServerRow.Expandable) -> Unit,
	onToggleFavorite: (String, Boolean) -> Unit,
	modifier: Modifier = Modifier,
) {
	val rotationAngle by animateFloatAsState(targetValue = if (row.isExpanded) 90f else 0f, label = "StateItemRotation")
	CountryDropDown(
		title = row.region,
		countryCode = row.locale.country.lowercase(),
		country = row.locale,
		rotationAngle = rotationAngle,
		expanded = row.isExpanded,
		gatewayCount = row.gatewayCount,
		isSelected = isSelected,
		isFavorite = row.isFavorite,
		shape = rowShape(roundTop = false, roundBottom = row.isLastInGroup),
		background = MaterialTheme.colorScheme.surface.copy(alpha = 0.7f),
		onToggleFavorite = { onToggleFavorite(row.region, row.isFavorite) },
		onDropDownClick = { onToggleExpanded(row) },
		onSelectionChange = { onSelectionChange(row.region) },
		modifier = modifier.padding(bottom = if (row.isLastInGroup) GroupSpacing else 0.dp),
	)
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
	gatewayCount: Int,
	shape: Shape,
	background: Color,
	onDropDownClick: () -> Unit,
	onSelectionChange: () -> Unit,
	onToggleFavorite: () -> Unit,
	modifier: Modifier = Modifier,
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
						"$gatewayCount ${stringResource(R.string.servers)}",
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
internal fun GatewayRow(
	row: ServerRow.Gateway,
	gatewayType: GatewayType,
	gatewayLocation: GatewayLocation,
	isSelected: Boolean,
	onSelectionChange: (String) -> Unit,
	onGatewayDetails: (NymGateway) -> Unit,
	onToggleFavorite: (String, Boolean) -> Unit,
	modifier: Modifier = Modifier,
) {
	val gateway = row.gateway
	val isFavorite = row.isFavorite
	SurfaceSelectionGroupButton(
		listOf(
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
						showStreamDisplay = gateway.showStreamDisplay(gatewayLocation),
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
				selected = isSelected,
			),
		),
		shape = rowShape(roundTop = false, roundBottom = row.isLastInGroup),
		background = MaterialTheme.colorScheme.surface.copy(alpha = 0.5f),
		divider = false,
		anchorsPadding = 0.dp,
		modifier = modifier.padding(bottom = if (row.isLastInGroup) GroupSpacing else 0.dp),
	)
}

internal fun NymGateway.showStreamDisplay(gatewayLocation: GatewayLocation): Boolean = gatewayLocation == GatewayLocation.EXIT && asnKind == AsnKind.RESIDENTIAL
