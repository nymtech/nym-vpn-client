package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun LogsTabBar(
	selectedTab: Int,
	onTabSelected: (Int) -> Unit
) {
	Box(
		modifier = Modifier
			.fillMaxWidth()
			.padding(vertical = 8.dp),
		contentAlignment = Alignment.Center
	) {
		Surface(
			tonalElevation = 4.dp,
			shadowElevation = 4.dp,
			shape = RoundedCornerShape(24.dp),
			color = MaterialTheme.colorScheme.surface,
		) {
			Row(
				modifier = Modifier
					.height(40.dp)
					.padding(horizontal = 8.dp),
				verticalAlignment = Alignment.CenterVertically
			) {
				LogsTabButton("Android Logs", isSelected = selectedTab == 0) { onTabSelected(0) }
				LogsTabButton("NymVPN Logs", isSelected = selectedTab == 1) { onTabSelected(1) }
			}
		}
	}
}
