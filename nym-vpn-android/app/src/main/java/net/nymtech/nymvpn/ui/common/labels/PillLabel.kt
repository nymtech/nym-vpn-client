package net.nymtech.nymvpn.ui.common.labels

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.util.scaledHeight

@Composable
fun PillLabel(text: String, backgroundColor: Color, textColor: Color) {
	Surface(
		modifier =
		Modifier
			.height(56.dp.scaledHeight())
			.width(159.dp),
		shape = RoundedCornerShape(size = 50.dp),
		color = backgroundColor,
	) {
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.SpaceEvenly,
		) {
			Text(
				text,
				textAlign = TextAlign.Center,
				color = textColor,
				style = MaterialTheme.typography.labelLarge,
			)
		}
	}
}
