package net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import net.nymtech.nymvpn.ui.common.buttons.OutlineStyledButton
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun SignUpView(onBackClick: () -> Unit, onAccountClick: () -> Unit, onSocialClick: () -> Unit, modifier: Modifier = Modifier) {
	Column(
		modifier = modifier
			.background(MaterialTheme.colorScheme.surface)
			.fillMaxWidth()
			.padding(horizontal = 18.dp, vertical = 16.dp),
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(22.dp),
	) {
		Box(modifier = Modifier.fillMaxWidth()) {
			IconButton(
				onClick = onBackClick,
				modifier = Modifier.align(Alignment.CenterStart).size(24.dp),
			) {
				Icon(
					imageVector = Icons.AutoMirrored.Filled.KeyboardArrowLeft,
					tint = MaterialTheme.colorScheme.onBackground,
					contentDescription = stringResource(R.string.button_back),
				)
			}
			Icon(
				imageVector = ImageVector.vectorResource(R.drawable.app_label),
				contentDescription = stringResource(R.string.app_name),
				tint = MaterialTheme.colorScheme.onPrimaryContainer,
				modifier = Modifier.align(Alignment.Center),
			)
		}

		Text(
			text = stringResource(R.string.auth_sign_up_title),
			style = MaterialTheme.typography.headlineSmall,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			modifier = Modifier.padding(20.dp),
		)

		MainStyledButton(
			onClick = onAccountClick,
			content = {
				Text(
					stringResource(R.string.auth_sign_up_account_button),
					style = MaterialTheme.typography.titleMedium,
				)
			},
			modifier = Modifier.fillMaxWidth().height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
		)

		OutlineStyledButton(
			onClick = onSocialClick,
			content = {
				Text(
					stringResource(R.string.auth_sign_up_social_button),
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
			},
			modifier = Modifier.fillMaxWidth().height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
		)

		Text(
			text = stringResource(R.string.auth_sign_up_info),
			textAlign = TextAlign.Center,
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onSurface,
			modifier = Modifier.fillMaxWidth(),
		)
	}
}

@Preview(name = "SignUpViewPreview", uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun PreviewSignUpViewDark() {
	NymVPNTheme(Theme.DARK_MODE) {
		SignUpView(
			onBackClick = {},
			onAccountClick = {},
			onSocialClick = {},
		)
	}
}
