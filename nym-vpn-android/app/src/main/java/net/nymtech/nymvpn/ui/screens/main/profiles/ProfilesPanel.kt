package net.nymtech.nymvpn.ui.screens.main.profiles

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
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
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
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import kotlin.enums.enumEntries

@Composable
fun ProfilesPanel(expanded: Boolean, selected: Profile?, anchorHeightPx: Int, onDismiss: () -> Unit, onSelect: (Profile) -> Unit) {
	val transitionState = remember { MutableTransitionState(false) }
	transitionState.targetState = expanded

	if (!transitionState.currentState && !transitionState.targetState) return

	val density = LocalDensity.current
	val startOffsetPx = with(density) { 16.dp.scaledWidth().roundToPx() }

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
			ProfilesPanelContent(selected = selected, onSelect = onSelect)
		}
	}
}

@Composable
private fun ProfilesPanelContent(selected: Profile?, onSelect: (Profile) -> Unit) {
	Surface(
		shape = RoundedCornerShape(16.dp),
		color = MaterialTheme.colorScheme.surface,
		shadowElevation = 18.dp,
	) {
		Column(modifier = Modifier.width(IntrinsicSize.Max)) {
			enumEntries<Profile>().forEach { profile ->
				ProfileRow(
					profile = profile,
					selected = profile == selected,
					onClick = { onSelect(profile) },
					modifier = Modifier.fillMaxWidth(),
				)
			}
		}
	}
}

@Composable
private fun ProfileRow(profile: Profile, selected: Boolean, onClick: () -> Unit, modifier: Modifier = Modifier) {
	val interactionSource = remember { MutableInteractionSource() }
	Row(
		modifier = modifier
			.background(
				color = if (selected) MaterialTheme.colorScheme.primaryContainer else Color.Transparent,
				shape = RoundedCornerShape(16.dp),
			)
			.clickable(interactionSource = interactionSource, indication = null, onClick = onClick)
			.padding(start = 10.dp, end = 12.dp, top = 14.dp, bottom = 14.dp),
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
				style = CustomTypography.titleMediumBold,
				color = if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onPrimaryContainer,
			)
			Text(
				text = stringResource(profile.descriptionRes),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onSurfaceVariant,
				modifier = Modifier.padding(top = 4.dp.scaledHeight()),
			)
		}
	}
}

@Composable
@PreviewLightDark
private fun ProfilesPanelPreview() {
	NymVPNTheme(Theme.default()) {
		ProfilesPanelContent(selected = Profile.SAFEST, onSelect = {})
	}
}
