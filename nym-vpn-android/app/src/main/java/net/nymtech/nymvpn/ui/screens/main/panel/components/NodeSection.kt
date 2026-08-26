package net.nymtech.nymvpn.ui.screens.main.panel.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.ripple
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.screens.details.components.CountryFlag
import net.nymtech.nymvpn.ui.screens.main.panel.NodeSelectionType
import net.nymtech.nymvpn.ui.screens.main.panel.ServerNode
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.getScoreIcon

@Composable
internal fun NodeSection(
	label: String,
	node: ServerNode,
	isClickable: Boolean,
	onNodeClick: () -> Unit,
	onInfoClick: () -> Unit,
	visible: Boolean,
	alwaysShowRow: Boolean,
	modifier: Modifier = Modifier,
) {
	Column(modifier = modifier) {
		AnimatedVisibility(
			visible = visible,
			enter = expandVertically(animationSpec = tween(350)) + fadeIn(animationSpec = tween(350)),
			exit = shrinkVertically(animationSpec = tween(350)) + fadeOut(animationSpec = tween(350)),
		) {
			Text(
				text = label,
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onSurfaceVariant,
				modifier = Modifier.padding(bottom = 8.dp),
			)
		}

		if (alwaysShowRow) {
			ServerRow(
				node = node,
				isClickable = isClickable,
				onServerClick = onNodeClick,
				onInfoClick = onInfoClick,
				modifier = Modifier.padding(bottom = 16.dp),
			)
		} else {
			AnimatedVisibility(
				visible = visible,
				enter = expandVertically(animationSpec = tween(350)) + fadeIn(animationSpec = tween(350)),
				exit = shrinkVertically(animationSpec = tween(350)) + fadeOut(animationSpec = tween(350)),
			) {
				ServerRow(
					node = node,
					isClickable = isClickable,
					onServerClick = onNodeClick,
					onInfoClick = onInfoClick,
					modifier = Modifier.padding(bottom = 16.dp),
				)
			}
		}
	}
}

@Composable
private fun ServerRow(node: ServerNode, isClickable: Boolean, onServerClick: () -> Unit, onInfoClick: () -> Unit, modifier: Modifier = Modifier) {
	val indication = if (isClickable) ripple() else null
	val showDetails = node.location != null
	val locationAlpha by animateFloatAsState(
		targetValue = if (showDetails) 1f else 0f,
		animationSpec = tween(350),
		label = "locationAlpha",
	)
	val locationLineHeight = with(LocalDensity.current) {
		MaterialTheme.typography.bodySmall.lineHeight.toDp()
	}
	val nameOffset by animateDpAsState(
		targetValue = if (showDetails) 0.dp else (locationLineHeight + 2.dp) / 2,
		animationSpec = tween(350),
		label = "nameOffset",
	)

	Row(
		verticalAlignment = Alignment.CenterVertically,
		horizontalArrangement = Arrangement.spacedBy(8.dp),
		modifier = modifier.fillMaxWidth(),
	) {
		// 'Safest' and 'Random' carry no gateway of their own — until the daemon
		// reports the one it picked (score arrives with it), CountryFlag's
		// selection icon stands alone instead of an unknown-score indicator.
		if (node.score != null || node.selectionType == NodeSelectionType.NODE) {
			val (icon, description) = getScoreIcon(node.score)
			Image(
				icon,
				contentDescription = description,
				modifier = Modifier.size(iconSize).padding(2.dp),
			)
		}

		Column(
			verticalArrangement = Arrangement.spacedBy(2.dp),
			modifier = Modifier
				.weight(1f)
				.clickable(interactionSource = remember { MutableInteractionSource() }, indication = indication) {
					if (isClickable) onServerClick()
				},
		) {
			Row(
				verticalAlignment = Alignment.CenterVertically,
				horizontalArrangement = Arrangement.spacedBy(8.dp),
				modifier = Modifier.offset(y = nameOffset),
			) {
				CountryFlag(node.countryCode, 22.dp, node.selectionType)

				Text(
					text = node.name.orEmpty(),
					style = MaterialTheme.typography.bodyLarge,
					color = if (isClickable) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurface,
					maxLines = 1,
					overflow = TextOverflow.Ellipsis,
					modifier = Modifier.weight(1f),
				)
			}

			Text(
				text = node.location.orEmpty(),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onSurface,
				maxLines = 1,
				overflow = TextOverflow.Ellipsis,
				modifier = Modifier.alpha(locationAlpha),
			)
		}

		if (showDetails) {
			Icon(
				imageVector = Icons.Outlined.Info,
				contentDescription = null,
				tint = MaterialTheme.colorScheme.primary,
				modifier = Modifier
					.size(26.dp)
					.clickable(
						interactionSource = remember { MutableInteractionSource() },
						indication = ripple(bounded = false),
						onClick = onInfoClick,
					),
			)
		}
	}
}
