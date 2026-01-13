package net.nymtech.nymvpn.ui.screens.onboarding.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp

@Composable
fun PagerIndicator(pageCount: Int, currentPage: Int, modifier: Modifier = Modifier) {
	Row(
		horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
		verticalAlignment = Alignment.CenterVertically,
		modifier = modifier,
	) {
		repeat(pageCount) { index ->
			val isSelected = index == currentPage
			val color = if (isSelected) MaterialTheme.colorScheme.onBackground else MaterialTheme.colorScheme.outline

			Spacer(
				modifier = Modifier
					.size(8.dp)
					.clip(CircleShape)
					.background(color),
			)
		}
	}
}
