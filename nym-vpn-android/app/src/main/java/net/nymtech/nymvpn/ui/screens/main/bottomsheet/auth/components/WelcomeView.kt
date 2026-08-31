package net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components

import PrivacyText
import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun WelcomeView(onLoginClick: () -> Unit, onSignUpClick: () -> Unit, modifier: Modifier = Modifier) {
	Column(
		modifier = modifier
			.background(MaterialTheme.colorScheme.surface)
			.fillMaxWidth()
			.padding(horizontal = 18.dp, vertical = 16.dp),
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(22.dp),
	) {
		Icon(
			imageVector = ImageVector.vectorResource(R.drawable.app_label),
			contentDescription = stringResource(R.string.app_name),
			tint = MaterialTheme.colorScheme.onPrimaryContainer,
		)
		Text(
			text = stringResource(R.string.auth_welcome_title),
			style = MaterialTheme.typography.headlineSmall,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
		)
		Text(
			text = stringResource(R.string.auth_welcome_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onSurface,
			textAlign = TextAlign.Center,
		)

		MainStyledButton(
			onClick = onSignUpClick,
			content = {
				Text(
					stringResource(R.string.auth_sign_up_title),
					style = MaterialTheme.typography.titleMedium,
				)
			},
			modifier = Modifier
				.fillMaxWidth()
				.height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
		)

		MainStyledButton(
			onClick = onLoginClick,
			content = {
				Text(
					stringResource(R.string.auth_welcome_login_button),
					style = MaterialTheme.typography.titleMedium,
				)
			},
			modifier = Modifier
				.fillMaxWidth()
				.height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
		)

		PrivacyText()
	}
}

@Preview(name = "WelcomeViewPreview", uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun PreviewWelcomeViewDark() {
	NymVPNTheme(Theme.DARK_MODE) {
		WelcomeView(
			onLoginClick = {},
			onSignUpClick = {},
		)
	}
}
