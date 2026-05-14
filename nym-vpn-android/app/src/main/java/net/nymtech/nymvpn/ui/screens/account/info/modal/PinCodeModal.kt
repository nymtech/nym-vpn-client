package net.nymtech.nymvpn.ui.screens.account.info.modal

import android.content.ClipData
import android.content.res.Configuration
import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Check
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.toClipEntry
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun PinCodeDialog(pinCode: String, url: String, onDismiss: () -> Unit) {
	val context = LocalContext.current
	val clipboard = LocalClipboard.current
	val scope = rememberCoroutineScope()
	var copied by remember { mutableStateOf(false) }

	LaunchedEffect(copied) {
		if (copied) {
			delay(3_000)
			copied = false
		}
	}

	AlertDialog(
		containerColor = MaterialTheme.colorScheme.surface,
		tonalElevation = 0.dp,
		onDismissRequest = onDismiss,
		icon = {
			Box(
				modifier = Modifier
					.size(48.dp)
					.clip(RoundedCornerShape(12.dp))
					.background(MaterialTheme.colorScheme.primary.copy(alpha = 0.15f)),
				contentAlignment = Alignment.Center,
			) {
				Icon(
					imageVector = Icons.Outlined.Lock,
					contentDescription = null,
					tint = MaterialTheme.colorScheme.primary,
					modifier = Modifier.size(24.dp),
				)
			}
		},
		title = {
			Text(
				text = stringResource(R.string.pin_code_title),
				style = CustomTypography.titleMediumPlus,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				textAlign = TextAlign.Center,
			)
		},
		text = {
			Column(horizontalAlignment = Alignment.CenterHorizontally) {
				Text(
					text = stringResource(R.string.pin_code_subtitle),
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					textAlign = TextAlign.Center,
				)

				Spacer(Modifier.height(24.dp))

				Row(
					horizontalArrangement = Arrangement.Center,
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier.fillMaxWidth(),
				) {
					pinCode.forEachIndexed { index, char ->
						if (index > 0) {
							Box(
								modifier = Modifier
									.padding(horizontal = 8.dp)
									.size(6.dp)
									.clip(CircleShape)
									.background(MaterialTheme.colorScheme.primary),
							)
						}
						Text(
							text = char.toString(),
							color = MaterialTheme.colorScheme.onPrimaryContainer,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
							style = MaterialTheme.typography.headlineLarge.copy(fontWeight = FontWeight.Bold),
						)
					}
				}
			}
		},
		confirmButton = {
			MainStyledButton(
				onClick = {
					scope.launch {
						val clip = ClipData.newPlainText("pin code", pinCode)
						clipboard.setClipEntry(clip.toClipEntry())
						context.openWebUrl(url)
						copied = true
					}
				},
				content = {
					AnimatedContent(copied, label = "copy_state") { isCopied ->
						Row(
							verticalAlignment = Alignment.CenterVertically,
							horizontalArrangement = Arrangement.spacedBy(8.dp),
						) {
							Icon(
								imageVector = if (isCopied) Icons.Outlined.Check else Icons.Outlined.ContentCopy,
								contentDescription = null,
								modifier = Modifier.size(18.dp),
							)
							Text(
								text = if (isCopied) {
									stringResource(R.string.pin_code_copied)
								} else {
									stringResource(R.string.pin_code_copy_and_open)
								},
								style = CustomTypography.buttonMain,
								fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
							)
						}
					}
				},
				color = MaterialTheme.colorScheme.primary,
				modifier = Modifier
					.fillMaxWidth()
					.height(48.dp.scaledHeight()),
			)
		},
	)
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun PreviewPinCodeDialog() {
	NymVPNTheme(Theme.default()) {
		PinCodeDialog(
			pinCode = "A1B2C3",
			url = "https://nym.com",
			onDismiss = {},
		)
	}
}
