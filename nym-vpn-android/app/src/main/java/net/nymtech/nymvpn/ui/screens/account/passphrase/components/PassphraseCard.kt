package net.nymtech.nymvpn.ui.screens.account.passphrase.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp

@Composable
fun PassphraseCard(modifier: Modifier = Modifier, onShowClick: () -> Unit = {}) {
	val shape = RoundedCornerShape(18.dp)

	Box(
		modifier
			.fillMaxWidth()
			.height(364.dp)
			.shadow(elevation = 6.dp, shape = shape, clip = false)
			.clip(shape)
			.background(MaterialTheme.colorScheme.surface)
			.border(BorderStroke(1.dp, MaterialTheme.colorScheme.outline), shape),
	) {
		OutlinedButton(
			onClick = onShowClick,
			border = BorderStroke(1.dp, MaterialTheme.colorScheme.onBackground),
			shape = RoundedCornerShape(24.dp),
			colors = ButtonDefaults.outlinedButtonColors(
				containerColor = Color.Transparent,
				contentColor = MaterialTheme.colorScheme.onBackground,
			),
			modifier = Modifier
				.align(Alignment.Center)
				.height(44.dp)
				.padding(horizontal = 24.dp),
		) {
			Icon(
				imageVector = Icons.Outlined.Visibility,
				contentDescription = "Show",
				modifier = Modifier.size(18.dp),
			)
			Spacer(Modifier.width(10.dp))
			Text(
				text = "Show my passphrase",
				style = MaterialTheme.typography.labelLarge.copy(
					fontWeight = FontWeight.SemiBold,
				),
			)
		}
	}
}

@Preview(showBackground = true)
@Composable
private fun PassphraseRevealCardPreview() {
	MaterialTheme(colorScheme = darkColorScheme()) {
		PassphraseCard()
	}
}
