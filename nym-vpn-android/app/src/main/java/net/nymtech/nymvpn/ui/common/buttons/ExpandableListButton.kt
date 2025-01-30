package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDropDown
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun ExpandableListButton(leadingIcon: ImageVector, title: @Composable () -> Unit, description: @Composable () -> Unit) {
	val interactionSource = remember { MutableInteractionSource() }
	var expanded by remember { mutableStateOf(false) }

	val rotationAngle by animateFloatAsState(targetValue = if (expanded) 180f else 0f)

	Column {
		Box(
			contentAlignment = Alignment.Center,
			modifier =
			Modifier
				.clickable(
					interactionSource = interactionSource,
					indication = null,
				) {
					// TODO
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
					leadingIcon.let { icon ->
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
							.padding(start = 16.dp.scaledWidth())
							.padding(vertical = 16.dp.scaledHeight()),
					) {
						title()
						description()
					}
				}
				Box(
					contentAlignment = Alignment.CenterEnd,
					modifier = Modifier
						.padding(end = 24.dp.scaledWidth(), start = 16.dp.scaledWidth())
						.weight(1f),
				) {
					Row {
						HorizontalDivider()
						val icon = Icons.Filled.ArrowDropDown
						IconButton(
							onClick = {
								expanded = !expanded
							},
						) {
							Icon(
								imageVector = icon,
								contentDescription = if (expanded) "Collapse" else "Expand",
								modifier = Modifier.graphicsLayer(rotationZ = rotationAngle),
							)
						}
					}
				}
			}
		}
		AnimatedVisibility(
			visible = expanded,
			enter = expandVertically() + fadeIn(),
			exit = shrinkVertically() + fadeOut(),
		) {
		}
	}
}
