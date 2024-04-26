package net.nymtech.nymvpn.ui.common.labels

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
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
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun PillLabel(text: String, backgroundColor: Color, textColor: Color, trailing: (@Composable () -> Unit)? = null) {
	Surface(
		modifier =
		Modifier
			.height(56.dp.scaledHeight())
			.width(IntrinsicSize.Min),
		shape = RoundedCornerShape(size = 50.dp),
		color = backgroundColor,
	) {
		Row(
			horizontalArrangement = Arrangement.spacedBy(5.dp, Alignment.CenterHorizontally),
			verticalAlignment = Alignment.CenterVertically,
			modifier = Modifier.padding(horizontal = 24.dp.scaledWidth()),
		) {
			Text(
				text,
				textAlign = TextAlign.Center,
				color = textColor,
				style = CustomTypography.labelHuge,
			)
			trailing?.let {
				trailing()
			}
		}
	}
}
