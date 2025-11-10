package net.nymtech.nymvpn.ui.screens.account.plan.components

import android.content.res.Configuration
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.billing.model.ProductData
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.addDaysToToday
import net.nymtech.nymvpn.util.extensions.scaledHeight

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SubscriptionBottomSheet(products: List<ProductData>, onDismiss: () -> Unit, onSelect: (ProductData) -> Unit) {
	ModalBottomSheet(
		onDismissRequest = onDismiss,
		shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
		containerColor = MaterialTheme.colorScheme.background,
		tonalElevation = 4.dp,
	) {
		SubscriptionBottomSheetContent(products, onDismiss, onSelect)
	}
}

@Composable
fun SubscriptionBottomSheetContent(products: List<ProductData>, onDismiss: () -> Unit, onSelect: (ProductData) -> Unit) {
	Column(
		modifier = Modifier
			.fillMaxWidth()
			.background(color = MaterialTheme.colorScheme.background)
			.padding(16.dp),
		verticalArrangement = Arrangement.spacedBy(12.dp),
	) {
		Text(
			text = stringResource(R.string.select_plan_modal_title),
			style = MaterialTheme.typography.titleLarge,
			color = MaterialTheme.colorScheme.onSurface,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			modifier = Modifier.align(Alignment.CenterHorizontally),
		)

		Spacer(Modifier.height(8.dp))

		Column(
			modifier = Modifier
				.clip(RoundedCornerShape(8.dp))
				.fillMaxWidth()
				.border(
					BorderStroke(1.dp, MaterialTheme.colorScheme.onSurface),
					RoundedCornerShape(8.dp),
				)
				.background(MaterialTheme.colorScheme.surface),
		) {
			products.forEachIndexed { index, product ->
				Column(
					modifier = Modifier
						.fillMaxWidth()
						.clickable { onSelect(product) }
						.padding(vertical = 24.dp, horizontal = 16.dp),
					verticalArrangement = Arrangement.spacedBy(4.dp),
				) {
					product.freeTrialDays?.let { _ ->
						Text(
							text = stringResource(R.string.select_plan_modal_free_trial),
							style = MaterialTheme.typography.titleMedium,
							color = MaterialTheme.colorScheme.onSurface,
						)
						Text(
							text = stringResource(R.string.select_plan_modal_free_trial_starts_today),
							style = MaterialTheme.typography.bodySmall,
							color = MaterialTheme.colorScheme.onSurfaceVariant,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
							modifier = Modifier.padding(bottom = 4.dp),
						)
					}
					Text(
						text = product.name + " (${product.price})",
						style = MaterialTheme.typography.titleMedium,
						color = MaterialTheme.colorScheme.onSurface,
					)
					product.freeTrialDays?.let { d ->
						Text(
							text = stringResource(R.string.select_plan_modal_free_trial_starts, d.addDaysToToday()),
							style = MaterialTheme.typography.bodySmall,
							color = MaterialTheme.colorScheme.onSurfaceVariant,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						)
					}
				}

				if (index < products.lastIndex) {
					HorizontalDivider(
						color = MaterialTheme.colorScheme.outline.copy(alpha = 0.5f),
						thickness = 1.dp,
					)
				}
			}
		}

		MainStyledButton(
			onClick = onDismiss,
			content = {
				Text(
					stringResource(R.string.cancel),
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					color = MaterialTheme.colorScheme.onSurface,
					style = MaterialTheme.typography.titleMedium,
				)
			},
			modifier = Modifier.fillMaxWidth().height(56.dp.scaledHeight()),
			color = MaterialTheme.colorScheme.surface,
			borderStroke = BorderStroke(width = 1.dp, color = MaterialTheme.colorScheme.onSurface),
		)
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SubscriptionBottomSheetContentPreview() {
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
			override val freeTrialDays = 7
		},
	)

	NymVPNTheme(Theme.default()) {
		SubscriptionBottomSheetContent(
			products = mockProducts,
			onDismiss = {},
			onSelect = {},
		)
	}
}
