package net.nymtech.nymvpn.ui.screens.settings.tunneling.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun FilterButton(title: String, noOfApps: Int, description: String, imageVector: ImageVector, isSelected: Boolean, modifier: Modifier = Modifier) {
	Card(
		modifier = modifier.heightIn(min = 56.dp),
		shape = RoundedCornerShape(8.dp.scaledHeight()),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.background),
		border = if (isSelected) BorderStroke(1.dp, MaterialTheme.colorScheme.onBackground) else null,
	) {
		Column(
			verticalArrangement = Arrangement.Center,
			modifier = Modifier
				.padding(horizontal = 12.dp.scaledWidth(), vertical = 8.dp.scaledHeight())
				.fillMaxHeight(),
		) {
			Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp.scaledWidth())) {
				Icon(
					imageVector = imageVector,
					contentDescription = null,
					modifier = Modifier.size(16.dp.scaledHeight()),
					tint = MaterialTheme.colorScheme.onBackground,
				)
				Text(
					text = title,
					style = MaterialTheme.typography.bodyMedium.copy(fontWeight = if (isSelected) FontWeight(700) else Typography.bodyMedium.fontWeight),
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					maxLines = 1,
					overflow = TextOverflow.Ellipsis,
					modifier = Modifier.weight(1f),
				)
				Text(
					text = "($noOfApps)",
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.onBackground,
					maxLines = 1,
					softWrap = false,
					modifier = Modifier.wrapContentWidth(),
				)
			}
			Spacer(modifier = Modifier.height(4.dp.scaledHeight()))
			Text(
				text = description,
				style = MaterialTheme.typography.bodySmall.copy(fontSize = 10.sp, lineHeight = 14.sp),
				color = if (isSelected) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onBackground,
			)
		}
	}
}
