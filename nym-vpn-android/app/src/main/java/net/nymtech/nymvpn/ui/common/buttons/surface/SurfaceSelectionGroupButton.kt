package net.nymtech.nymvpn.ui.common.buttons.surface

import android.content.res.Configuration
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SurfaceSelectionGroupButton(
	items: List<SelectionItem>,
	shape: Shape = RoundedCornerShape(14.dp),
	background: Color,
	divider: Boolean = true,
	anchorsPadding: Dp = 16.dp,
	modifier: Modifier = Modifier,
) {
	val interactionSource = remember { MutableInteractionSource() }
	Card(
		modifier = modifier.fillMaxWidth().wrapContentHeight(),
		shape = shape,
		colors = CardDefaults.cardColors(containerColor = background),
	) {
		items.mapIndexed { index, it ->
			Box(
				contentAlignment = Alignment.Center,
				modifier =
				Modifier
					.clickable(
						interactionSource = interactionSource,
						indication = null,
					) {
						it.onClick()
					}
					.fillMaxWidth()
					.border(
						width = 1.dp,
						color = if (it.selected) MaterialTheme.colorScheme.primary else Color.Transparent,
						shape = RoundedCornerShape(14.dp),
					)
					.padding(horizontal = 8.dp, vertical = 6.dp),
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier.fillMaxWidth(),
				) {
					Row(
						verticalAlignment = Alignment.CenterVertically,
						modifier = Modifier
							.weight(1f, false)
							.padding(end = 4.dp.scaledWidth()),
					) {
						Box(modifier = Modifier.padding(start = anchorsPadding.scaledWidth()))
						it.leading?.let { icon ->
							Box(modifier = Modifier.padding(end = anchorsPadding.scaledWidth())) {
								icon()
							}
						}
						Column(
							horizontalAlignment = Alignment.Start,
							verticalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterVertically),
							modifier = Modifier
								.fillMaxWidth()
								.padding(vertical = if (it.description == null) 16.dp.scaledHeight() else 6.dp.scaledHeight()),
						) {
							it.title()
							it.description?.let {
								it()
							}
						}
					}
					it.trailing?.let { trailing ->
						Box(
							contentAlignment = Alignment.CenterEnd,
							modifier = Modifier
								.wrapContentWidth(Alignment.End)
								.padding(start = 8.dp, end = anchorsPadding.scaledWidth()),
						) {
							trailing()
						}
					}
				}
			}
			if (index + 1 != items.size && divider) HorizontalDivider(color = MaterialTheme.colorScheme.surfaceVariant)
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
private fun PreviewSurfaceSelectionGroupButton() {
	NymVPNTheme(Theme.default()) {
		SurfaceSelectionGroupButton(
			background = MaterialTheme.colorScheme.surface,
			items = listOf(
				SelectionItem(
					leading = { Icon(Icons.Filled.Settings, contentDescription = null) },
					trailing = { Switch(checked = true, onCheckedChange = null) },
					title = { Text("Auto-connect", style = MaterialTheme.typography.bodyLarge) },
					description = { Text("Connect on startup", style = MaterialTheme.typography.bodySmall) },
					selected = true,
				),
				SelectionItem(
					leading = { Icon(Icons.Filled.Info, contentDescription = null) },
					trailing = { Switch(checked = false, onCheckedChange = null) },
					title = { Text("Bypass LAN", style = MaterialTheme.typography.bodyLarge) },
					selected = false,
				),
			),
		)
	}
}
