package net.nymtech.nymvpn.ui.screens.account.passphrase

import android.app.Activity
import android.content.ClipData
import android.content.Intent
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
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
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import androidx.fragment.app.FragmentActivity
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarEvent
import net.nymtech.nymvpn.ui.screens.account.passphrase.components.PassphraseActions
import net.nymtech.nymvpn.ui.screens.account.passphrase.components.PassphraseCard
import net.nymtech.nymvpn.ui.screens.account.passphrase.modal.PassphraseInfo
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.DeviceAuthHelper
import net.nymtech.nymvpn.util.extensions.savePasswordToManager
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
	)
}

@Composable
fun PassphraseScreen(passphrase: List<String>, show: Boolean, onShowClick: () -> Unit, onCopyClick: () -> Unit, onDownloadClick: () -> Unit, onSaveClick: () -> Unit) {
	Column(
		modifier = Modifier
			.fillMaxSize()
			.imePadding()
			.verticalScroll(rememberScrollState())
			.padding(horizontal = 16.dp.scaledWidth(), vertical = 24.dp),
	) {
		Text(
			text = stringResource(R.string.passphrase_title),
			style = MaterialTheme.typography.bodyLarge,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			modifier = Modifier.fillMaxWidth(),
		)

		Text(
			text = buildAnnotatedString {
				withStyle(style = SpanStyle(color = MaterialTheme.colorScheme.onBackground)) {
					append(stringResource(R.string.passphrase_description_first))
				}
				append(" ")
				withStyle(
					style = SpanStyle(
						color = MaterialTheme.colorScheme.onBackground,
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
		)
	}
}
