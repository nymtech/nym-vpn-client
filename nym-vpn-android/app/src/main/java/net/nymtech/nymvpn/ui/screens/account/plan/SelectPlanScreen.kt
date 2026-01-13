package net.nymtech.nymvpn.ui.screens.account.plan

import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Campaign
import androidx.compose.material.icons.filled.VerifiedUser
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.rememberVectorPainter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.billing.model.ProductData
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.account.plan.components.SubscriptionBottomSheet
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import timber.log.Timber

@Composable
fun SelectPlanScreen(appUiState: AppUiState, viewModel: SelectPlanViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val products by viewModel.subscriptions.collectAsState()
	var showSheet by remember { mutableStateOf(false) }
	val navController = LocalNavController.current

	SelectPlanScreen(
		products = products,
		showSheet = showSheet,
		onSelectPlanButtonClick = {
			if (viewModel.isBillingAvailable()) {
				viewModel.fetchSubscriptions()
				showSheet = true
			} else {
				appUiState.managerState.accountLinks?.signUp?.let {
					Timber.d("Create url: $it")
					context.openWebUrl(it)
				}
			}
		},
		onDismissSheet = { showSheet = false },
		onSelectSubscription = { product ->
			showSheet = false
			navController.navigate(Route.Payment(product.id))
		},
	)
}

@Composable
private fun FilledProgressBar4(modifier: Modifier = Modifier) {
	Row(
		modifier = modifier
			.fillMaxWidth()
			.height(4.dp),
		horizontalArrangement = Arrangement.spacedBy(4.dp),
	) {
		repeat(4) {
			Box(
				modifier = Modifier
					.weight(1f)
					.fillMaxHeight()
					.background(
						MaterialTheme.colorScheme.primary,
						shape = RoundedCornerShape(4.dp),
					),
			)
		}
	}
}

@Composable
fun SelectPlanScreen(
	products: List<ProductData> = emptyList(),
	showSheet: Boolean = false,
	onSelectPlanButtonClick: () -> Unit,
	onDismissSheet: () -> Unit,
	onSelectSubscription: (ProductData) -> Unit,
	padding: PaddingValues = WindowInsets.systemBars.asPaddingValues(),
) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = Modifier
			.fillMaxSize()
			.imePadding()
			.background(MaterialTheme.colorScheme.background)
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		FilledProgressBar4(
			modifier = Modifier.padding(top = 8.dp, bottom = 24.dp),
		)

		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.CenterVertically),
			modifier = Modifier
				.padding(vertical = 24.dp.scaledHeight())
				.weight(1f),
		) {
			Box(
				modifier = Modifier
					.border(width = 1.dp, color = CustomColors.iconBorder, shape = RoundedCornerShape(size = 8.dp))
					.background(color = CustomColors.iconBackground, shape = RoundedCornerShape(size = 8.dp))
					.padding(start = 12.dp, top = 12.dp, end = 12.dp, bottom = 12.dp),
				contentAlignment = Alignment.Center,
			) {
				Image(
					painter = rememberVectorPainter(Icons.Rounded.Check),
					contentDescription = null,
					modifier = Modifier
						.clip(CircleShape)
						.background(MaterialTheme.colorScheme.primary)
						.size(40.dp)
						.padding(4.dp),
				)
			}
			Text(
				text = stringResource(R.string.select_plan_title),
				style = MaterialTheme.typography.headlineSmall,
				color = MaterialTheme.colorScheme.onSurface,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)

			Column(
				verticalArrangement = Arrangement.spacedBy(24.dp, Alignment.CenterVertically),
				modifier = Modifier.padding(horizontal = 52.dp).fillMaxWidth(),
			) {
				Row {
					Icon(
						painter = rememberVectorPainter(Icons.Filled.VerifiedUser),
						contentDescription = null,
						tint = MaterialTheme.colorScheme.primary,
						modifier = Modifier.size(16.dp),
					)
					Spacer(Modifier.width(4.dp))
					Text(
						text = stringResource(R.string.select_plan_line_1),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.outline,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				}
				Row {
					Icon(
						painter = rememberVectorPainter(Icons.Filled.Campaign),
						contentDescription = null,
						tint = MaterialTheme.colorScheme.primary,
						modifier = Modifier.size(16.dp),
					)
					Spacer(Modifier.width(4.dp))
					Text(
						text = stringResource(R.string.select_plan_line_2),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.outline,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				}
				Row {
					Icon(
						painter = painterResource(R.drawable.ic_chat_error),
						contentDescription = null,
						tint = MaterialTheme.colorScheme.primary,
						modifier = Modifier.size(16.dp),
					)
					Spacer(Modifier.width(6.dp))
					Text(
						text = stringResource(R.string.select_plan_line_3),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.outline,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				}
			}
		}
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Bottom),
			modifier = Modifier.padding(vertical = 24.dp.scaledHeight()),
		) {
			MainStyledButton(
				onClick = onSelectPlanButtonClick,
				content = {
					Text(
						stringResource(R.string.select_plan_button),
						style = CustomTypography.buttonMain,
					)
				},
				color = MaterialTheme.colorScheme.primary,
				modifier = Modifier
					.fillMaxWidth()
					.height(56.dp.scaledHeight()),
			)
		}
	}

	if (showSheet && products.isNotEmpty()) {
		SubscriptionBottomSheet(
			products = products,
			onDismiss = onDismissSheet,
			onSelect = onSelectSubscription,
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewSelectPlanScreen() {
	NymVPNTheme(Theme.default()) {
		val mockProducts = listOf(
			object : ProductData {
				override val id = "1"
				override val name = "Monthly Plan"
				override val price = "$4.99 / month"
				override val freeTrialDays = null
			},
			object : ProductData {
				override val id = "2"
				override val name = "Yearly Plan"
				override val price = "$49.99 / year"
				override val freeTrialDays = null
			},
		)
		SelectPlanScreen(products = mockProducts, onSelectPlanButtonClick = {}, onSelectSubscription = {}, onDismissSheet = {})
	}
}
