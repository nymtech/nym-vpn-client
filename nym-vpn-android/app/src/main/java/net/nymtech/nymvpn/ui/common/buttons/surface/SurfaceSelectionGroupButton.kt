package net.nymtech.nymvpn.ui.common.buttons.surface

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun SurfaceSelectionGroupButton(items: List<SelectionItem>) {
	val interactionSource = remember { MutableInteractionSource() }
	Card(
		modifier = Modifier.fillMaxWidth(),
		shape = RoundedCornerShape(8.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
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
					.fillMaxWidth(),
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp.scaledHeight()),
				) {
					Row(
						verticalAlignment = Alignment.CenterVertically,
						modifier = Modifier
							.padding(start = 16.dp.scaledWidth())
							.weight(4f, false)
							.fillMaxWidth(),
					) {
						it.leadingIcon?.let { icon ->
							Icon(
								icon,
								icon.name,
								modifier = Modifier.size(iconSize.scaledWidth()),
							)
						}
						Column(
							horizontalAlignment = Alignment.Start,
							verticalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterVertically),
							modifier = Modifier
								.fillMaxWidth()
								.padding(start = if (it.leadingIcon != null) 16.dp.scaledWidth() else 0.dp)
								.padding(vertical = if (it.description == null) 16.dp.scaledHeight() else 6.dp.scaledHeight()),
						) {
							it.title()
							it.description?.let {
								it()
							}
						}
					}
					it.trailing?.let {
						Box(
							contentAlignment = Alignment.CenterEnd,
							modifier = Modifier
								.padding(end = 24.dp.scaledWidth(), start = 16.dp.scaledWidth())
								.weight(1f),
						) {
							it()
						}
					}
						?: Box(
							contentAlignment = Alignment.CenterEnd,
							modifier = Modifier
								.padding(end = 12.dp.scaledWidth())
								.weight(1f),
						) {
							val icon = ImageVector.vectorResource(R.drawable.link_arrow_right)
							Icon(
								icon,
								icon.name,
								Modifier.size(
									iconSize,
								),
							)
						}
				}
			}
			if (index + 1 != items.size) HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
		}
	}
}
