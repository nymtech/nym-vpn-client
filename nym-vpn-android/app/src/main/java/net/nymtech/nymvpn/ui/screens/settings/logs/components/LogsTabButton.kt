package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun LogsTabButton(label: String, isSelected: Boolean, onClick: () -> Unit) {
	val backgroundColor = if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surface
	val contentColor = if (isSelected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface

	Surface(
		modifier = Modifier
			.padding(horizontal = 4.dp)
			.height(32.dp),
		shape = RoundedCornerShape(20.dp),
		color = backgroundColor,
		tonalElevation = if (isSelected) 2.dp else 0.dp,
		shadowElevation = if (isSelected) 2.dp else 0.dp,
		onClick = onClick,
	) {
		Box(
			modifier = Modifier
				.padding(horizontal = 16.dp),
			contentAlignment = Alignment.Center,
		) {
			Text(
				text = label,
				color = contentColor,
				style = MaterialTheme.typography.labelMedium,
			)
		}
	}
}
