package net.nymtech.nymvpn.ui.screens.account.info.components

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun AccountButton(
	text: String,
	onClick: () -> Unit,
	modifier: Modifier = Modifier,
	textColor: Color = MaterialTheme.colorScheme.onPrimaryContainer,
	containerColor: Color = MaterialTheme.colorScheme.primaryContainer,
	trailingIcon: ImageVector? = null,
) {
	MainStyledButton(
		onClick = onClick,
		content = {
			Row(
				horizontalArrangement = Arrangement.spacedBy(8.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				Text(
					text,
					style = MaterialTheme.typography.titleMedium,
					color = textColor,
				)
				trailingIcon?.let {
					Icon(
						imageVector = it,
						contentDescription = null,
						tint = textColor,
						modifier = Modifier.size(18.dp),
					)
				}
			}
		},
		color = containerColor,
		textColor = textColor,
		modifier = modifier.fillMaxWidth().height(48.dp.scaledHeight()),
		shape = RoundedCornerShape(12.dp),
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewAccountButton() {
	NymVPNTheme(Theme.default()) {
		Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
			AccountButton(text = "Pair new device", onClick = {})
			AccountButton(text = "Log out", onClick = {}, textColor = LocalNymColors.current.buttonErrorText)
			AccountButton(
				text = "Delete account",
				onClick = {},
				textColor = LocalNymColors.current.buttonErrorText,
				containerColor = Color.Transparent,
				trailingIcon = Icons.Outlined.Lock,
			)
		}
	}
}
