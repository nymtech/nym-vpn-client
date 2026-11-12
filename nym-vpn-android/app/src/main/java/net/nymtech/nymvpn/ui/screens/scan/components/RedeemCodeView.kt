package net.nymtech.nymvpn.ui.screens.scan.components

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.functions.SecureScreen
import net.nymtech.nymvpn.ui.common.security.SecureOverlayProtection
import net.nymtech.nymvpn.ui.screens.scan.RedeemCodeUiState
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

// Laid out like the auth views, for the welcome sheet and the redeem sheet.
@Composable
fun RedeemCodeView(uiState: RedeemCodeUiState, onCodeChange: (String) -> Unit, onSubmit: () -> Unit, modifier: Modifier = Modifier, onBackClick: (() -> Unit)? = null) {
	SecureScreen()
	SecureOverlayProtection()

	Column(
		modifier = modifier
			.background(MaterialTheme.colorScheme.surface)
			.fillMaxWidth()
			.imePadding()
			.padding(horizontal = 18.dp, vertical = 16.dp),
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(22.dp),
	) {
		Box(modifier = Modifier.fillMaxWidth()) {
			if (onBackClick != null) {
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
			}
			Icon(
				imageVector = ImageVector.vectorResource(R.drawable.app_label),
				contentDescription = stringResource(R.string.app_name),
				tint = MaterialTheme.colorScheme.onPrimaryContainer,
				modifier = Modifier.align(Alignment.Center),
			)
		}
		Text(
			text = stringResource(R.string.redeem_code_title),
			style = MaterialTheme.typography.headlineSmall,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			modifier = Modifier.semantics { heading() },
		)
		RedeemCodeForm(uiState = uiState, onCodeChange = onCodeChange, onSubmit = onSubmit)
	}
}

@Preview(name = "RedeemCodeViewPreview", uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun PreviewRedeemCodeViewDark() {
	NymVPNTheme(Theme.DARK_MODE) {
		RedeemCodeView(
			uiState = RedeemCodeUiState(),
			onBackClick = {},
			onCodeChange = {},
			onSubmit = {},
		)
	}
}
