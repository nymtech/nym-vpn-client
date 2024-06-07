package net.nymtech.nymvpn.ui.common.labels

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun PermissionLabel(selectionItem: SelectionItem) {
	Card(
		modifier = Modifier.fillMaxWidth(),
		shape = RoundedCornerShape(8.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.background),
	) {
		Box(
			contentAlignment = Alignment.Center,
			modifier =
			Modifier.fillMaxWidth()
				.width(IntrinsicSize.Max)
				.height(IntrinsicSize.Min)
				.padding(vertical = 8.dp.scaledHeight()),
		) {
			Row(
				verticalAlignment = Alignment.CenterVertically,
				horizontalArrangement = Arrangement.SpaceBetween,
				modifier = Modifier.fillMaxWidth(),
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					horizontalArrangement = Arrangement.spacedBy(16.dp.scaledWidth()),
					modifier = Modifier.padding(start = 16.dp.scaledWidth()),
				) {
					selectionItem.leadingIcon?.let { icon ->
						Icon(
							icon,
							icon.name,
							modifier = Modifier.size(iconSize.scaledWidth()),
						)
					}
					Column(
						horizontalAlignment = Alignment.Start,
						verticalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterVertically),
						modifier = Modifier.fillMaxWidth(),
					) {
						selectionItem.title()
						selectionItem.description?.let { it() }
					}
					selectionItem.trailing?.let {
						Box(
							contentAlignment = Alignment.CenterEnd,
							modifier = Modifier
								.padding(start = 16.dp.scaledWidth(), end = 24.dp.scaledWidth()),
						) {
							it()
						}
					}
				}
			}
		}
	}
}
