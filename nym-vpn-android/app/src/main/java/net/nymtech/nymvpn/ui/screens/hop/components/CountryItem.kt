package net.nymtech.nymvpn.ui.screens.hop.components

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
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
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
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.getFlagImageVectorByName
import net.nymtech.nymvpn.util.extensions.getScoreIcon
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib.GatewayType
import java.util.*

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
