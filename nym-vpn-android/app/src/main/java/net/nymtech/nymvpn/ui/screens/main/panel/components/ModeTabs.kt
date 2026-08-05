package net.nymtech.nymvpn.ui.screens.main.panel.components

import android.content.res.Configuration
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Spacer
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectMode
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.getModeIcon

internal val ConnectMode.labelRes: Int
	get() = when (this) {
		ConnectMode.FAST -> R.string.one_click_mode_fast
		ConnectMode.MIXNET -> R.string.connect_mode_mixnet
	}

@Composable
internal fun ModeTabs(selected: ConnectMode, onSelect: (ConnectMode) -> Unit, modifier: Modifier = Modifier) {
	val modes = ConnectMode.entries
	var selectedIndex = modes.indexOf(selected)
	val indicatorPadding = 2.dp

	BoxWithConstraints(
		modifier = modifier
			.fillMaxWidth()
			.height(50.dp)
			.background(color = MaterialTheme.colorScheme.surfaceVariant, shape = RoundedCornerShape(27.dp))
			.padding(indicatorPadding),
	) {
		val tabWidth = maxWidth / modes.size
		val indicatorOffset by animateDpAsState(
			targetValue = tabWidth * selectedIndex,
			animationSpec = spring(dampingRatio = 0.82f, stiffness = 600f),
			label = "modeTabIndicator",
		)

		Box(
			modifier = Modifier
				.offset(x = indicatorOffset)
				.width(tabWidth)
				.fillMaxHeight()
				.clip(RoundedCornerShape(23.dp))
				.background(MaterialTheme.colorScheme.surface),
		)

		Row(modifier = Modifier.fillMaxSize()) {
			modes.forEach { mode ->
				val textColor by animateColorAsState(
					targetValue = if (selected == mode) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface,
					animationSpec = tween(200),
					label = "modeTabColor_${mode.name}",
				)
				Row(
					horizontalArrangement = Arrangement.Center,
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier
						.width(tabWidth)
						.fillMaxHeight()
						.clickable(
							interactionSource = remember { MutableInteractionSource() },
							indication = null,
						) { onSelect(mode) },
				) {
					Icon(
						imageVector = getModeIcon(mode),
						contentDescription = null,
						tint = textColor,
						modifier = Modifier.size(20.dp),
					)
					Spacer(modifier = Modifier.width(4.dp))
					Text(
						text = stringResource(mode.labelRes),
						style = MaterialTheme.typography.labelLarge.copy(
							fontWeight = if (selected == mode) FontWeight.Bold else FontWeight.Normal,
						),
						color = textColor,
						maxLines = 1,
					)
				}
			}
		}
	}
}

@Preview(name = "Fast selected – dark", uiMode = Configuration.UI_MODE_NIGHT_YES, showBackground = true)
@Preview(name = "Fast selected – light", uiMode = Configuration.UI_MODE_NIGHT_NO, showBackground = true)
@Composable
private fun PreviewModeTabs() {
	NymVPNTheme(Theme.DARK_MODE) {
		var selected by remember { mutableStateOf(ConnectMode.FAST) }
		ModeTabs(
			selected = selected,
			onSelect = { selected = it },
			modifier = Modifier.padding(16.dp),
		)
	}
}
