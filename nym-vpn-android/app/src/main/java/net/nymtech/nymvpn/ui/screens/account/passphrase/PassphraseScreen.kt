package net.nymtech.nymvpn.ui.screens.account.passphrase

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
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
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.credentials.CreatePasswordRequest
import androidx.credentials.CredentialManager
import androidx.fragment.app.FragmentActivity
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.account.passphrase.components.PassphraseActions
import net.nymtech.nymvpn.ui.screens.account.passphrase.components.PassphraseCard
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.safePopBackStack
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import timber.log.Timber

@Composable
fun PassphraseScreen(hideBackButton: (Boolean) -> Unit, viewModel: PassphraseViewModel = hiltViewModel()) {
	val clipboardManager = LocalClipboardManager.current
	val passphrase by viewModel.passphrase.collectAsState()
	var showSheet by remember { mutableStateOf(false) }
	val navController = LocalNavController.current
	val context = LocalContext.current
	val scope = rememberCoroutineScope()
	val activity = context as? FragmentActivity
	val executor = remember(context) { ContextCompat.getMainExecutor(context) }

	val biometricPrompt = remember(activity, executor) {
		activity?.let {
			BiometricPrompt(
				it,
				executor,
				object : BiometricPrompt.AuthenticationCallback() {
					override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
						showSheet = true
					}
				},
			)
		}
	}

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

	val promptInfo = remember(context) { buildPromptInfo(context) }

	// Prevent system back click navigation if passphrase sheet is visible
	BackHandler(enabled = showSheet) { }

	LaunchedEffect(showSheet) {
		hideBackButton(showSheet)
	}

	fun requestAuthOrReveal() {
		val manager = BiometricManager.from(context)

		val authenticators =
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
				BiometricManager.Authenticators.BIOMETRIC_STRONG or
					BiometricManager.Authenticators.DEVICE_CREDENTIAL
			} else {
				BiometricManager.Authenticators.BIOMETRIC_STRONG
			}

		when (manager.canAuthenticate(authenticators)) {
			BiometricManager.BIOMETRIC_SUCCESS,
			BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED,
			-> {
				if (biometricPrompt != null) {
					biometricPrompt.authenticate(promptInfo)
				} else {
					showSheet = true
				}
			}

			BiometricManager.BIOMETRIC_ERROR_NO_HARDWARE,
			BiometricManager.BIOMETRIC_ERROR_HW_UNAVAILABLE,
			BiometricManager.BIOMETRIC_ERROR_SECURITY_UPDATE_REQUIRED,
			BiometricManager.BIOMETRIC_ERROR_UNSUPPORTED,
			BiometricManager.BIOMETRIC_STATUS_UNKNOWN,
			-> {
				showSheet = true
			}

			else -> showSheet = true
		}
	}

	suspend fun savePasswordToManager(context: Context, password: String) {
		val credentialManager = CredentialManager.create(context)

		val passwordCredential = CreatePasswordRequest(
			id = "nym-passphrase",
			password = password,
		)

		try {
			credentialManager.createCredential(
				request = passwordCredential,
				context = context,
			)
		} catch (e: Exception) {
			Timber.d(e)
		}
	}

	PassphraseScreen(
		passphrase = passphrase,
		show = showSheet,
		onShowClick = {
			requestAuthOrReveal()
		},
		onCopyClick = {
			clipboardManager.setText(AnnotatedString(passphrase.joinToString(" ")))
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
				savePasswordToManager(
					context = context,
					password = passphrase.joinToString(" "),
				)
			}
		},
		onContinueClick = {
			navController.safePopBackStack()
		},
	)
}

@Suppress("DEPRECATION")
private fun buildPromptInfo(context: Context): BiometricPrompt.PromptInfo {
	val title = context.getString(R.string.passphrase_title)
	val subtitle = context.getString(R.string.passphrase_description)

	return when {
		Build.VERSION.SDK_INT >= Build.VERSION_CODES.R -> {
			BiometricPrompt.PromptInfo.Builder()
				.setTitle(title)
				.setSubtitle(subtitle)
				.setAllowedAuthenticators(
					BiometricManager.Authenticators.BIOMETRIC_STRONG or
						BiometricManager.Authenticators.DEVICE_CREDENTIAL,
				)
				.build()
		}

		Build.VERSION.SDK_INT == Build.VERSION_CODES.Q -> {
			BiometricPrompt.PromptInfo.Builder()
				.setTitle(title)
				.setSubtitle(subtitle)
				.setDeviceCredentialAllowed(true)
				.build()
		}

		else -> {
			BiometricPrompt.PromptInfo.Builder()
				.setTitle(title)
				.setSubtitle(subtitle)
				.setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_STRONG)
				.setNegativeButtonText(context.getString(android.R.string.cancel))
				.build()
		}
	}
}

@Composable
fun PassphraseScreen(
	passphrase: List<String>,
	show: Boolean,
	onShowClick: () -> Unit,
	onCopyClick: () -> Unit,
	onDownloadClick: () -> Unit,
	onSaveClick: () -> Unit,
	onContinueClick: () -> Unit,
) {
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
			modifier = Modifier
				.fillMaxWidth(),
		)
		Text(
			text = buildAnnotatedString {
				withStyle(
					style = SpanStyle(
						color = MaterialTheme.colorScheme.outline,
					),
				) {
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
					onClick = {
						if (confirmed) {
							onContinueClick()
						}
					},
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
