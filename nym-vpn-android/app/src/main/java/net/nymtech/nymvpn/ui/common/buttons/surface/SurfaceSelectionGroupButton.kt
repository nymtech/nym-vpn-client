package net.nymtech.nymvpn.ui.common.buttons.surface

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SurfaceSelectionGroupButton(
	items: List<SelectionItem>,
	shape: Shape = RoundedCornerShape(8.dp),
	background: Color,
	divider: Boolean = true,
	anchorsPadding: Dp = 16.dp,
	modifier: Modifier = Modifier,
) {
	val interactionSource = remember { MutableInteractionSource() }
	Card(
		modifier = modifier.fillMaxWidth(),
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
					.fillMaxWidth().height(IntrinsicSize.Min),
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier.fillMaxSize(),
				) {
					if (it.selected) {
						Box(
							modifier = Modifier
								.offset(x = 0.dp, y = 0.dp)
								.width(4.dp)
								.fillMaxHeight()
								.background(
									color = MaterialTheme.colorScheme.primary,
									shape = RoundedCornerShape(topStart = 0.dp, topEnd = 4.dp, bottomStart = 0.dp, bottomEnd = 4.dp),
								),
						)
					}
					Row(
						verticalAlignment = Alignment.CenterVertically,
						modifier = Modifier
							.weight(4f, false)
							.padding(vertical = 4.dp.scaledHeight()).fillMaxSize(),
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
								.padding(horizontal = anchorsPadding.scaledWidth())
								.weight(1f),
						) {
							trailing()
						}
					}
				}
			}
			if (index + 1 != items.size && divider) HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
		}
	}
}
