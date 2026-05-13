package net.nymtech.nymvpn.ui.screens.account.info.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsIcon
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsTitle

@Composable
fun AccountInfoCard(title: String, value: String, icon: ImageVector, onClick: () -> Unit) {
	Card(
		modifier = Modifier.fillMaxWidth().clickable {
			onClick.invoke()
		},
		shape = RoundedCornerShape(8.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
	) {
		Column(modifier = Modifier.fillMaxWidth()) {
			Row(
				modifier = Modifier.padding(vertical = 14.dp, horizontal = 16.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				SettingsIcon(icon, "")
				Spacer(Modifier.width(18.dp))
				SettingsTitle(title)
			}
			HorizontalDivider(color = MaterialTheme.colorScheme.surfaceVariant)
			Row(
				modifier = Modifier
					.fillMaxWidth().padding(vertical = 14.dp, horizontal = 16.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				Text(
					text = value,
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground,
					modifier = Modifier.weight(1f),
				)
				Icon(
					imageVector = Icons.Outlined.ContentCopy,
					contentDescription = stringResource(R.string.go),
					modifier = Modifier.size(16.dp),
					tint = MaterialTheme.colorScheme.onBackground,
				)
			}
		}
	}
}
