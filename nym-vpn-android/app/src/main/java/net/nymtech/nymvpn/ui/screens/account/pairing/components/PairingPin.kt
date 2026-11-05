package net.nymtech.nymvpn.ui.screens.account.pairing.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.nymtech.nymvpn.R

@Composable
fun PairingPin(pin: String, modifier: Modifier = Modifier) {
	val spokenPin = pin.toCharArray().joinToString(" ")
	val description = stringResource(R.string.pairing_pin_description, spokenPin)
	Text(
		text = pin.chunked(PIN_GROUP_SIZE).joinToString(" "),
		style = MaterialTheme.typography.headlineLarge,
		fontFamily = FontFamily.Monospace,
		fontWeight = FontWeight.SemiBold,
		letterSpacing = 4.sp,
		color = MaterialTheme.colorScheme.onSurface,
		modifier = modifier
			.background(MaterialTheme.colorScheme.surfaceContainer, RoundedCornerShape(12.dp))
			.padding(horizontal = 24.dp, vertical = 16.dp)
			.clearAndSetSemantics { contentDescription = description },
	)
}

private const val PIN_GROUP_SIZE = 3
