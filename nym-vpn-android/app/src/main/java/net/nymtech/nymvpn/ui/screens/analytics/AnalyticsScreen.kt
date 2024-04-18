package net.nymtech.nymvpn.ui.screens.analytics

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.ClickableText
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Analytics
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun AnalyticsScreen(navController: NavController, appViewModel: AppViewModel, appUiState: AppUiState) {
	val errorReportingDescription = buildAnnotatedString {
		append("(")
		append(stringResource(id = R.string.via))
		append(" ")
		pushStringAnnotation(tag = "sentry", annotation = stringResource(id = R.string.sentry_url))
		withStyle(style = SpanStyle(color = MaterialTheme.colorScheme.primary)) {
			append(stringResource(id = R.string.sentry))
		}
		pop()
		append("), ")
		append(stringResource(id = R.string.required_app_restart))
	}

	val analyticsMessage = buildAnnotatedString {
		append(stringResource(id = R.string.alpha_help_message))
		append(" ")
		append(stringResource(id = R.string.opt_in_message_first))
		append(" ")
		append("(")
		append(stringResource(id = R.string.via))
		append(" ")
		pushStringAnnotation(tag = "sentry", annotation = stringResource(id = R.string.sentry_url))
		withStyle(style = SpanStyle(color = MaterialTheme.colorScheme.primary)) {
			append(stringResource(id = R.string.sentry))
		}
		pop()
		append(") ")
		append(stringResource(id = R.string.opt_in_message_second))
	}

	val termsMessage = buildAnnotatedString {
		append(stringResource(id = R.string.continue_agree))
		append(" ")
		pushStringAnnotation(tag = "terms", annotation = stringResource(id = R.string.terms_link))
		withStyle(style = SpanStyle(color = MaterialTheme.colorScheme.onBackground)) {
			append(stringResource(id = R.string.terms_of_use))
		}
		pop()
		append(" ")
		append(stringResource(id = R.string.and_acknowledge))
		append(" ")
		pushStringAnnotation(tag = "privacy", annotation = stringResource(id = R.string.privacy_link))
		withStyle(style = SpanStyle(color = MaterialTheme.colorScheme.onBackground)) {
			append(stringResource(id = R.string.privacy_policy))
		}
		pop()
		append(".")
	}

	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(60.dp.scaledHeight(), Alignment.Bottom),
		modifier =
		Modifier
			.fillMaxSize()
			.padding(horizontal = 16.dp.scaledWidth()),
	) {
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(16.dp.scaledHeight(), Alignment.Bottom),
			modifier = Modifier.padding(horizontal = 40.dp.scaledWidth()),
		) {
			Image(
				painter = painterResource(id = R.drawable.login),
				contentDescription = stringResource(id = R.string.login),
				contentScale = ContentScale.None,
				modifier =
				Modifier
					.width(80.dp)
					.height(80.dp),
			)
			Text(
				stringResource(id = R.string.welcome) + "\r\n" + stringResource(id = R.string.to_alpha),
				style = MaterialTheme.typography.headlineSmall.copy(color = MaterialTheme.colorScheme.onBackground),
			)
			ClickableText(
				text = analyticsMessage,
				style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center),
			) {
				analyticsMessage.getStringAnnotations(tag = "sentry", it, it).firstOrNull()?.let { annotation ->
					appViewModel.openWebPage(annotation.item)
				}
			}
		}
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(24.dp, Alignment.Top),
		) {
			SurfaceSelectionGroupButton(
				items = listOf(
					SelectionItem(
						Icons.Outlined.BugReport,
						title = {
							Text(
								stringResource(R.string.error_reporting),
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onSurface),
							)
						},
						description = {
							ClickableText(
								text = errorReportingDescription,
								style = MaterialTheme.typography.bodySmall.copy(color = MaterialTheme.colorScheme.onSurfaceVariant),
							) {
								errorReportingDescription.getStringAnnotations(tag = "sentry", it, it).firstOrNull()?.let { annotation ->
									appViewModel.openWebPage(annotation.item)
								}
							}
						},
						height = 80,
						trailing = {
							Switch(
								appUiState.settings.errorReportingEnabled,
								{ appViewModel.onErrorReportingSelected() },
								modifier =
								Modifier
									.height(32.dp.scaledHeight())
									.width(52.dp.scaledWidth()),
							)
						},
					),
					SelectionItem(
						Icons.Outlined.Analytics,
						title = {
							Text(stringResource(R.string.share_anonymous_analytics), style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onSurface))
						},
						trailing = {
							Switch(
								appUiState.settings.analyticsEnabled,
								{ appViewModel.onAnalyticsReportingSelected() },
								modifier =
								Modifier
									.height(32.dp.scaledHeight())
									.width(52.dp.scaledWidth()),
							)
						},
						height = 80,
					),
				),
			)
			MainStyledButton(onClick = {
				appViewModel.setAnalyticsShown()
				navController.navigate(NavItem.Main.route) {
					// clear backstack after login
					popUpTo(0)
				}
			}, content = {
				Text(stringResource(id = R.string.cont), style = MaterialTheme.typography.labelLarge.copy(color = MaterialTheme.colorScheme.onPrimary))
			})
			ClickableText(
				text = termsMessage,
				style = MaterialTheme.typography.labelMedium.copy(color = MaterialTheme.colorScheme.outline, textAlign = TextAlign.Center),
				modifier = Modifier.padding(bottom = 24.dp.scaledHeight()),
			) {
				termsMessage.getStringAnnotations(tag = "terms", it, it).firstOrNull()?.let { annotation ->
					appViewModel.openWebPage(annotation.item)
				}
				termsMessage.getStringAnnotations(tag = "privacy", it, it).firstOrNull()?.let { annotation ->
					appViewModel.openWebPage(annotation.item)
				}
			}
		}
	}
}
