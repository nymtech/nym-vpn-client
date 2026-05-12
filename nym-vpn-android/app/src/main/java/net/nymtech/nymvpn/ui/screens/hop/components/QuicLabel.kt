package net.nymtech.nymvpn.ui.screens.hop.components

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.LocalNymColors

@Composable
internal fun QuickLabel() {
	val colors = LocalNymColors.current
	Row(
		modifier = Modifier
			.border(width = 1.dp, color = colors.borderCyan, shape = RoundedCornerShape(4.dp))
			.padding(horizontal = 8.dp, vertical = 4.dp),
		horizontalArrangement = Arrangement.spacedBy(2.dp),
		verticalAlignment = Alignment.CenterVertically,
	) {
		Icon(
			imageVector = ImageVector.vectorResource(R.drawable.quic_label),
			contentDescription = stringResource(R.string.quic),
			tint = Color.Unspecified,
		)
		Text(
			text = stringResource(R.string.quic),
			style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp, fontWeight = FontWeight.Bold),
			fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
			color = colors.labelCyan,
			maxLines = 1,
		)
	}
}
