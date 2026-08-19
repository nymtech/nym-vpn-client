package net.nymtech.nymvpn.ui.common.navigation.profile

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.MutableTransitionState
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

private val PanelCornerRadius = 22.dp
private val PanelWidth = 300.dp
private val RowCornerRadius = 14.dp
private val RowHeight = 75.dp
private val PanelAnchorStartOffset = 16.dp

@Composable
fun ProfilesPanel(expanded: Boolean, selected: Profile?, anchorHeightPx: Int, onDismiss: () -> Unit, onSelect: (Profile) -> Unit) {
	val transitionState = remember { MutableTransitionState(false) }
	transitionState.targetState = expanded

	if (!transitionState.currentState && !transitionState.targetState) return

	val density = LocalDensity.current
	val startOffsetPx = with(density) { PanelAnchorStartOffset.scaledWidth().roundToPx() }

	Popup(
		alignment = Alignment.TopStart,
		offset = IntOffset(startOffsetPx, anchorHeightPx),
		onDismissRequest = onDismiss,
		properties = PopupProperties(focusable = true),
	) {
		AnimatedVisibility(
			visibleState = transitionState,
			enter = expandVertically(expandFrom = Alignment.Top) + fadeIn(),
			exit = shrinkVertically(shrinkTowards = Alignment.Top) + fadeOut(),
		) {
			Surface(
				modifier = Modifier.width(PanelWidth.scaledWidth()),
				shape = RoundedCornerShape(PanelCornerRadius),
				color = MaterialTheme.colorScheme.surface,
				shadowElevation = 18.dp,
			) {
				Column {
					enumValues<Profile>().forEach { profile ->
						ProfileRow(
							profile = profile,
							selected = profile == selected,
							onClick = { onSelect(profile) },
						)
					}
				}
			}
		}
	}
}

@Composable
private fun ProfileRow(profile: Profile, selected: Boolean, onClick: () -> Unit) {
	val interactionSource = remember { MutableInteractionSource() }
	Row(
		modifier = Modifier
			.fillMaxWidth()
			.height(RowHeight.scaledHeight())
			.background(
				color = if (selected) LocalNymColors.current.statusConnectedBg else Color.Transparent,
				shape = RoundedCornerShape(RowCornerRadius),
			)
			.clickable(interactionSource = interactionSource, indication = null, onClick = onClick)
			.padding(horizontal = 12.dp.scaledWidth()),
		verticalAlignment = Alignment.CenterVertically,
	) {
		Box(modifier = Modifier.size(40.dp.scaledWidth()), contentAlignment = Alignment.Center) {
			Icon(
				painter = painterResource(profile.icon),
				contentDescription = null,
				tint = if (selected) MaterialTheme.colorScheme.primary else LocalNymColors.current.navBarIconTint,
				modifier = Modifier.size(iconSize),
			)
		}
		Column(modifier = Modifier.padding(start = 14.dp.scaledWidth())) {
			Text(
				text = stringResource(profile.titleRes),
				style = MaterialTheme.typography.titleMedium,
				color = if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onPrimaryContainer,
			)
			Text(
				text = stringResource(profile.descriptionRes),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onSurfaceVariant,
				modifier = Modifier.padding(top = 4.dp.scaledHeight()),
			)
		}
	}
}
