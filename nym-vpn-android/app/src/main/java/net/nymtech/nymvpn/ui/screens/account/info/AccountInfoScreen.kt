package net.nymtech.nymvpn.ui.screens.account.info

import android.content.ClipData
import android.content.res.Configuration
import androidx.compose.material3.Icon
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.Launch
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material.icons.outlined.Devices
import androidx.compose.material.icons.outlined.Numbers
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.toClipEntry
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import timber.log.Timber

@Composable
fun AccountInfoScreen(appUiState: AppUiState) {
	val context = LocalContext.current
	val scope = rememberCoroutineScope()
	val clipboardManager = LocalClipboard.current
	val accountId = appUiState.managerState.accountId ?: ""
	val deviceId = appUiState.managerState.deviceId ?: ""
	AccountInfoScreen(
		accountId = accountId,
		deviceId = deviceId,
		onAccountClick = {
			appUiState.managerState.accountLinks?.account?.let {
				Timber.d("Account url: $it")
				context.openWebUrl(it)
			}
		},
		onAccountIdClick = {
			appUiState.managerState.accountId?.let {
				scope.launch {
					val clipData = ClipData.newPlainText(it, it)
					clipboardManager.setClipEntry(clipData.toClipEntry())
				}
			}
		},
		onDeviceIdClick = {
			appUiState.managerState.deviceId?.let {
				scope.launch {
					val clipData = ClipData.newPlainText(it, it)
					clipboardManager.setClipEntry(clipData.toClipEntry())
				}
			}
		},
	)
}

@Composable
fun AccountInfoScreen(accountId: String, deviceId: String, onAccountClick: () -> Unit, onAccountIdClick: () -> Unit, onDeviceIdClick: () -> Unit) {
	val interactionSource = remember { MutableInteractionSource() }
	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier = Modifier
			.fillMaxSize()
			.padding(horizontal = 16.dp.scaledWidth(), vertical = 24.dp),
	) {
		Card(
			modifier = Modifier.fillMaxWidth(),
			shape = RoundedCornerShape(8.dp),
			colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
		) {
			Box(
				contentAlignment = Alignment.Center,
				modifier =
				Modifier
					.clickable(
						interactionSource = interactionSource,
						indication = null,
					) {
						onAccountClick.invoke()
					}
					.fillMaxWidth().height(IntrinsicSize.Min),
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier.fillMaxSize(),
				) {
					Row(
						verticalAlignment = Alignment.CenterVertically,
						modifier = Modifier
							.weight(1f, false)
							.padding(vertical = 8.dp.scaledHeight())
							.fillMaxSize()
							.padding(end = 4.dp.scaledWidth()),
					) {
						Box(modifier = Modifier.padding(start = 16.dp.scaledWidth()))
						Box(modifier = Modifier.padding(end = 16.dp.scaledWidth())) {
							Icon(
								Icons.Outlined.Person,
								stringResource(R.string.account),
								modifier = Modifier.size(24.dp.scaledWidth()),
								tint = MaterialTheme.colorScheme.outline,
							)
						}
						Column(
							horizontalAlignment = Alignment.Start,
							verticalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterVertically),
							modifier = Modifier
								.fillMaxWidth()
								.padding(vertical = 16.dp.scaledHeight()),
						) {
							Text(
								stringResource(R.string.account_on_nym),
								style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
							)
						}
					}

					Box(
						contentAlignment = Alignment.CenterEnd,
						modifier = Modifier
							.weight(0.35f)
							.padding(horizontal = 16.dp.scaledWidth()),
					) {
						Icon(
							Icons.AutoMirrored.Outlined.Launch,
							stringResource(R.string.go),
							modifier = Modifier.size(24.dp),
							tint = MaterialTheme.colorScheme.outline,
						)
					}
				}
			}
		}

		Card(
			modifier = Modifier.fillMaxWidth(),
			shape = RoundedCornerShape(8.dp),
			colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
		) {
			Box(
				contentAlignment = Alignment.Center,
				modifier =
				Modifier
					.clickable(
						interactionSource = interactionSource,
						indication = null,
					) {
						onAccountIdClick.invoke()
					}
					.fillMaxWidth(),
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier.fillMaxWidth(),
				) {
					Row(
						verticalAlignment = Alignment.CenterVertically,
						modifier = Modifier
							.weight(1f)
							.padding(vertical = 8.dp.scaledHeight()),
					) {
						Box(modifier = Modifier.padding(start = 16.dp.scaledWidth()))
						Column(
							horizontalAlignment = Alignment.Start,
							modifier = Modifier
								.fillMaxWidth()
								.padding(vertical = 16.dp.scaledHeight()),
						) {
							Row(
								verticalAlignment = Alignment.CenterVertically,
							) {
								Icon(
									Icons.Outlined.Numbers,
									stringResource(R.string.account_id_title),
									modifier = Modifier.size(24.dp.scaledWidth()),
									tint = MaterialTheme.colorScheme.outline,
								)
								Text(
									stringResource(R.string.account_id_title),
									style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
									modifier = Modifier.padding(start = 10.dp),
								)
							}
							Text(
								accountId,
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
								modifier = Modifier.padding(top = 10.dp),
							)
						}
					}

					Box(
						contentAlignment = Alignment.CenterEnd,
						modifier = Modifier
							.padding(horizontal = 16.dp.scaledWidth()),
					) {
						Icon(
							Icons.Outlined.ContentCopy,
							stringResource(R.string.go),
							modifier = Modifier.size(20.dp),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					}
				}
			}
		}

		Card(
			modifier = Modifier.fillMaxWidth(),
			shape = RoundedCornerShape(8.dp),
			colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
		) {
			Box(
				contentAlignment = Alignment.Center,
				modifier =
				Modifier
					.clickable(
						interactionSource = interactionSource,
						indication = null,
					) {
						onDeviceIdClick.invoke()
					}
					.fillMaxWidth(),
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier.fillMaxWidth(),
				) {
					Row(
						verticalAlignment = Alignment.CenterVertically,
						modifier = Modifier
							.weight(1f)
							.padding(vertical = 8.dp.scaledHeight()),
					) {
						Box(modifier = Modifier.padding(start = 16.dp.scaledWidth()))
						Column(
							horizontalAlignment = Alignment.Start,
							modifier = Modifier
								.fillMaxWidth()
								.padding(vertical = 16.dp.scaledHeight()),
						) {
							Row(
								verticalAlignment = Alignment.CenterVertically,
							) {
								Icon(
									Icons.Outlined.Devices,
									stringResource(R.string.account_device_id_title),
									modifier = Modifier.size(24.dp.scaledWidth()),
									tint = MaterialTheme.colorScheme.outline,
								)
								Text(
									stringResource(R.string.account_device_id_title),
									style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
									modifier = Modifier.padding(start = 10.dp),
								)
							}
							Text(
								deviceId,
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
								modifier = Modifier.padding(top = 10.dp),
							)
						}
					}

					Box(
						contentAlignment = Alignment.CenterEnd,
						modifier = Modifier
							.padding(horizontal = 16.dp.scaledWidth()),
					) {
						Icon(
							Icons.Outlined.ContentCopy,
							stringResource(R.string.go),
							modifier = Modifier.size(20.dp),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					}
				}
			}
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewAccountInfoScreen() {
	NymVPNTheme(Theme.default()) {
		AccountInfoScreen("2WigVJR14ZqfBnnqVyyPY3XPwqhkpb2W94igVJR1784Z3qfBnnqVyyP", "2WigVJR14ZqfBnnqVyyPY3XPwqhkpb2W94igVJR1784Z3qfBnnqVyyP", {}, {}, {})
	}
}
