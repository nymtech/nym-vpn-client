package net.nymtech.nymvpn.ui.screens.settings.geoexclusion.components

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CornerSize
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Warning
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.theme.LocalNymColors

private val WarningCardShape = RoundedCornerShape(
	topStart = CornerSize(0.dp),
	bottomStart = CornerSize(0.dp),
	topEnd = CornerSize(12.dp),
	bottomEnd = CornerSize(12.dp),
)

@Composable
fun WarningCard(text: String) {
	val warningColor = LocalNymColors.current.warning
	Card(
		shape = WarningCardShape,
		colors = CardDefaults.cardColors(containerColor = LocalNymColors.current.warningBackground),
		modifier = Modifier
			.fillMaxWidth()
			.drawWithContent {
				drawContent()
				drawRect(
					color = warningColor,
					topLeft = Offset(0f, 0f),
					size = Size(width = 4.dp.toPx(), height = size.height),
				)
			},
	) {
		Row(
			modifier = Modifier
				.padding(vertical = 12.dp, horizontal = 14.dp)
				.fillMaxWidth(),
			verticalAlignment = Alignment.CenterVertically,
		) {
			Icon(
				imageVector = Icons.Outlined.Warning,
				contentDescription = null,
				tint = warningColor,
				modifier = Modifier.size(24.dp),
			)
			Spacer(Modifier.width(10.dp))
			Text(
				text = text,
				color = warningColor,
				style = MaterialTheme.typography.bodyMedium,
			)
		}
	}
}
