package net.nymtech.nymvpn.ui.common.labels

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun PillLabel(text: String, backgroundColor: Color, textColor: Color, trailing: (@Composable () -> Unit)? = null) {
	Surface(
		modifier =
		Modifier
			.height(56.dp.scaledHeight())
			.wrapContentWidth(),
		shape = RoundedCornerShape(size = 50.dp),
		color = backgroundColor,
	) {
		Row(
			horizontalArrangement = Arrangement.spacedBy(5.dp, Alignment.CenterHorizontally),
			verticalAlignment = Alignment.CenterVertically,
			modifier = Modifier.padding(horizontal = 24.dp.scaledWidth()),
		) {
			Text(
				text.uppercase(),
				textAlign = TextAlign.Center,
				color = textColor,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
				maxLines = 1,
				style = CustomTypography.labelHuge,
			)
			trailing?.let {
				trailing()
			}
		}
	}
}
