package net.nymtech.nymvpn.ui.common.functions

import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle

fun boldMarkupText(raw: String): AnnotatedString = buildAnnotatedString {
	raw.split("**").forEachIndexed { index, segment ->
		if (index % 2 == 1) {
			withStyle(SpanStyle(fontWeight = FontWeight.Bold)) { append(segment) }
		} else {
			append(segment)
		}
	}
}
