package net.nymtech.nymvpn.ui.screens.settings.diagnostic

import android.content.res.Configuration
import android.widget.Toast
import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.animations.SpinningIcon
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.OutlineStyledButton
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun DiagnosticScreen(viewModel: DiagnosticViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val context = LocalContext.current
	val clipboardManager = LocalClipboardManager.current
	val copiedText = stringResource(R.string.diagnostic_copied)

	DiagnosticScreen(
		uiState = uiState,
		onRunClick = { viewModel.runDiagnostics() },
		onShareClick = { viewModel.shareReport(context) },
		onCopyClick = {
			uiState.report?.let {
				clipboardManager.setText(AnnotatedString(it))
				Toast.makeText(context, copiedText, Toast.LENGTH_SHORT).show()
			}
		},
	)
}

@Composable
fun DiagnosticScreen(uiState: DiagnosticUiState, onRunClick: () -> Unit, onShareClick: () -> Unit, onCopyClick: () -> Unit) {
	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(14.dp.scaledHeight(), Alignment.Top),
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(rememberScrollState())
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		MainStyledButton(
			onClick = onRunClick,
			enabled = !uiState.isLoading,
			content = {
				if (uiState.isLoading) {
					SpinningIcon(Icons.Outlined.Refresh, stringResource(R.string.diagnostic_run_button))
				} else {
					Text(
						text = stringResource(R.string.diagnostic_run_button),
						style = MaterialTheme.typography.titleMedium,
					)
				}
			},
			modifier = Modifier
				.fillMaxWidth()
				.height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
		)

		uiState.report?.let {
			OutlineStyledButton(
				onClick = onShareClick,
				content = {
					Text(
						text = stringResource(R.string.diagnostic_share_button),
						style = MaterialTheme.typography.titleMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
				},
				borderColor = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier
					.fillMaxWidth()
					.height(48.dp.scaledHeight()),
				shape = RoundedCornerShape(12.dp),
			)
		}

		uiState.error?.let { error ->
			Text(
				text = "${stringResource(R.string.diagnostic_error)}: $error",
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.error,
			)
		}

		uiState.report?.let { report ->
			Column(verticalArrangement = Arrangement.spacedBy(8.dp.scaledHeight())) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					horizontalArrangement = Arrangement.SpaceBetween,
					modifier = Modifier.fillMaxWidth(),
				) {
					Text(
						text = stringResource(R.string.diagnostic_report_title),
						style = MaterialTheme.typography.titleSmall,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
					IconButton(onClick = onCopyClick) {
						Icon(
							Icons.Outlined.ContentCopy,
							contentDescription = stringResource(R.string.diagnostic_copy_report),
							modifier = Modifier.size(20.dp),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					}
				}

				Box(
					modifier = Modifier
						.fillMaxWidth()
						.heightIn(min = 120.dp, max = 400.dp)
						.clip(RoundedCornerShape(8.dp))
						.background(MaterialTheme.colorScheme.primaryContainer)
						.verticalScroll(rememberScrollState())
						.horizontalScroll(rememberScrollState())
						.padding(12.dp),
				) {
					SelectionContainer {
						Text(
							text = report,
							style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
							color = MaterialTheme.colorScheme.onBackground,
						)
					}
				}
			}
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewDiagnosticScreen() {
	NymVPNTheme(Theme.default()) {
		DiagnosticScreen(
			uiState = DiagnosticUiState(
				report = "{\n  \"dns\": null,\n  \"http\": null,\n  \"gateway\": null\n}",
			),
			onRunClick = {},
			onShareClick = {},
			onCopyClick = {},
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewDiagnosticScreenLoading() {
	NymVPNTheme(Theme.default()) {
		DiagnosticScreen(
			uiState = DiagnosticUiState(isLoading = true),
			onRunClick = {},
			onShareClick = {},
			onCopyClick = {},
		)
	}
}
