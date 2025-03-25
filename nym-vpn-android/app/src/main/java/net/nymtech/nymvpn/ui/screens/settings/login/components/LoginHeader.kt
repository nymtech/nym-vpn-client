package net.nymtech.nymvpn.ui.screens.settings.login.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun LoginHeader() {
	Icon(
		imageVector = ImageVector.vectorResource(R.drawable.app_label),
		contentDescription = stringResource(R.string.login),
	)
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Top),
		modifier = Modifier.padding(vertical = 24.dp.scaledHeight()),
	) {
		Text(
			text = stringResource(R.string.welcome_to_nym).uppercase(),
			style = MaterialTheme.typography.headlineSmall,
			color = MaterialTheme.colorScheme.onBackground,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
		)
		Text(
			text = stringResource(R.string.enter_access_code),
			style = MaterialTheme.typography.bodyLarge,
			color = MaterialTheme.colorScheme.onSurfaceVariant,
			textAlign = TextAlign.Center,
		)
	}
}
