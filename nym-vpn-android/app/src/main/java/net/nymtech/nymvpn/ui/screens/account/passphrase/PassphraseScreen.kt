package net.nymtech.nymvpn.ui.screens.account.passphrase

import android.app.Activity
import android.content.ClipData
import android.content.Intent
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.toClipEntry
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import androidx.fragment.app.FragmentActivity
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarEvent
import net.nymtech.nymvpn.ui.screens.account.passphrase.components.PassphraseActions
import net.nymtech.nymvpn.ui.screens.account.passphrase.components.PassphraseCard
import net.nymtech.nymvpn.ui.screens.account.passphrase.modal.PassphraseInfo
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.DeviceAuthHelper
import net.nymtech.nymvpn.util.extensions.safePopBackStack
import net.nymtech.nymvpn.util.extensions.savePasswordToManager
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import timber.log.Timber

@Composable
fun PassphraseScreen(onBackButtonVisibilityChange: (Boolean) -> Unit, navBarEvent: NavBarEvent?, onNavBarEventConsume: () -> Unit, viewModel: PassphraseViewModel = hiltViewModel()) {
	val clipboardManager = LocalClipboard.current
	val passphrase by viewModel.passphrase.collectAsState()
	var showSheet by remember { mutableStateOf(false) }

	var showInfoModal by remember { mutableStateOf(false) }

	LaunchedEffect(navBarEvent) {
		if (navBarEvent == NavBarEvent.PassphraseInfoClicked) {
			showInfoModal = true
			onNavBarEventConsume()
		}
	}

	PassphraseInfo(
		show = showInfoModal,
		onDismiss = { showInfoModal = false },
	)

	val navController = LocalNavController.current
	val context = LocalContext.current
	val scope = rememberCoroutineScope()
	val activity = context as? FragmentActivity

	val fileSaverLauncher = rememberLauncherForActivityResult(
		contract = ActivityResultContracts.StartActivityForResult(),
	) { result ->
		if (result.resultCode == Activity.RESULT_OK) {
			result.data?.data?.also { uri ->
				try {
					context.contentResolver.openOutputStream(uri)?.use { outputStream ->
						outputStream.write(passphrase.joinToString().toByteArray())
					}
				} catch (e: Exception) {
					Timber.e(e, "Failed to write passphrase to file.")
				}
			}
		}
	}

	val title = stringResource(R.string.passphrase_title)
	val subtitle = stringResource(R.string.passphrase_description)

	val promptInfo = remember(context) {
		DeviceAuthHelper.buildPromptInfo(context, title = title, subtitle = subtitle)
	}

	LaunchedEffect(showSheet) {
		onBackButtonVisibilityChange(showSheet)
	}

	fun requestAuthOrReveal() {
		if (activity == null) {
			showSheet = true
			return
		}

		DeviceAuthHelper.authenticate(
			activity = activity,
			promptInfo = promptInfo,
			onAuthenticated = { showSheet = true },
			onUnavailable = { showSheet = true },
			onError = { _, _ ->
			},
		)
	}

	PassphraseScreen(
		passphrase = passphrase,
		show = showSheet,
		onShowClick = { requestAuthOrReveal() },
		onCopyClick = {
			scope.launch {
				val text = passphrase.joinToString(" ")
				val clipData = ClipData.newPlainText(text, text)
				clipboardManager.setClipEntry(clipData.toClipEntry())
			}
		},
		onDownloadClick = {
			val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
				addCategory(Intent.CATEGORY_OPENABLE)
				type = "text/plain"
				putExtra(Intent.EXTRA_TITLE, "nym-passphrase.txt")
			}
			fileSaverLauncher.launch(intent)
		},
		onSaveClick = {
			scope.launch {
				savePasswordToManager(context = context, password = passphrase.joinToString(" "))
			}
		},
		onContinueClick = { navController.safePopBackStack() },
	)
}

@Composable
fun PassphraseScreen(passphrase: List<String>, show: Boolean, onShowClick: () -> Unit, onCopyClick: () -> Unit, onDownloadClick: () -> Unit, onSaveClick: () -> Unit, onContinueClick: () -> Unit) {
	var confirmed by remember { mutableStateOf(false) }
	Column(
		modifier = Modifier
			.fillMaxSize()
			.imePadding()
			.verticalScroll(rememberScrollState())
			.padding(horizontal = 16.dp.scaledWidth(), vertical = 24.dp),
	) {
		Text(
			text = stringResource(R.string.passphrase_title),
			style = Typography.titleMedium,
			color = MaterialTheme.colorScheme.onBackground,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			modifier = Modifier.fillMaxWidth(),
		)

		Text(
			text = buildAnnotatedString {
				withStyle(style = SpanStyle(color = MaterialTheme.colorScheme.outline)) {
					append(stringResource(R.string.passphrase_description_first))
				}
				append(" ")
				withStyle(
					style = SpanStyle(
						color = MaterialTheme.colorScheme.outline,
						fontWeight = FontWeight.Bold,
					),
				) {
					append(stringResource(R.string.passphrase_description_last))
				}
			},
			modifier = Modifier.padding(top = 16.dp),
			style = MaterialTheme.typography.bodyMedium,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)

		PassphraseCard(passphrase = passphrase, show = show, onShowClick = onShowClick)
		PassphraseActions(show = show, onCopyClick = onCopyClick, onDownloadClick = onDownloadClick, onSaveClick = onSaveClick)

		if (show) {
			Spacer(modifier = Modifier.weight(1f))
			Column(
				modifier = Modifier
					.fillMaxWidth()
					.padding(top = 24.dp),
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					horizontalArrangement = Arrangement.spacedBy(16.dp),
					modifier = Modifier
						.fillMaxWidth()
						.clickable { confirmed = !confirmed },
				) {
					Checkbox(
						checked = confirmed,
						onCheckedChange = { confirmed = it },
						modifier = Modifier.size(20.dp),
					)
					Text(
						text = stringResource(R.string.passphrase_saved),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onSurface,
						textAlign = TextAlign.Start,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				}
				MainStyledButton(
					onClick = { if (confirmed) onContinueClick() },
					content = {
						Text(
							stringResource(R.string.welcome_continue),
							style = CustomTypography.buttonMain,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						)
					},
					color = MaterialTheme.colorScheme.primary.copy(if (confirmed) 1f else 0.5f),
					modifier = Modifier
						.fillMaxWidth()
						.padding(vertical = 16.dp)
						.height(54.dp.scaledHeight()),
				)
			}
		}
	}
}

@Composable
@PreviewLightDark
internal fun PreviewPassphraseScreen() {
	NymVPNTheme(Theme.default()) {
		PassphraseScreen(
			passphrase = listOf(
				"smoke", "fire", "water", "earth", "air", "joker", "thunder", "tornado",
				"hailstorm", "earthquake", "tsunami", "blizzard", "whisper", "ocean", "sparkle", "mystery",
				"echo", "dream", "sapphire", "horizon", "crimson", "vortex", "chroma", "draw",
			),
			show = true,
			onShowClick = {},
			onCopyClick = {},
			onDownloadClick = {},
			onSaveClick = {},
			onContinueClick = {},
		)
	}
}
