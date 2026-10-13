package net.nymtech.nymvpn.ui.screens.account.plan.components

import android.content.res.Configuration
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.billing.model.ProductData
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.billing.TrialEligibility
import net.nymtech.nymvpn.ui.common.NymModalBottomSheet
import net.nymtech.nymvpn.ui.common.buttons.OutlineStyledButton
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SubscriptionBottomSheet(products: List<ProductData>, trialEligibility: TrialEligibility, onDismiss: () -> Unit, onSelect: (ProductData) -> Unit) {
	NymModalBottomSheet(
		onDismissRequest = onDismiss,
		shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
		containerColor = MaterialTheme.colorScheme.surface,
		tonalElevation = 4.dp,
	) {
		SubscriptionBottomSheetContent(products, trialEligibility, onDismiss, onSelect)
	}
}

/** A loader replaces the list until eligibility resolves, so rows never grow and plans can't be picked on a stale answer. */
@Composable
fun SubscriptionBottomSheetContent(products: List<ProductData>, trialEligibility: TrialEligibility, onDismiss: () -> Unit, onSelect: (ProductData) -> Unit) {
	Column(
		modifier = Modifier
			.fillMaxWidth()
			.background(color = MaterialTheme.colorScheme.surface)
			.padding(16.dp),
		verticalArrangement = Arrangement.spacedBy(14.dp),
	) {
		Text(
			text = stringResource(R.string.select_plan_modal_title),
			style = MaterialTheme.typography.titleMedium,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			modifier = Modifier.align(Alignment.CenterHorizontally),
		)

		when (trialEligibility) {
			TrialEligibility.Loading -> Box(
				contentAlignment = Alignment.Center,
				modifier = Modifier
					.fillMaxWidth()
					.height(PRODUCT_ROW_HEIGHT * products.size),
			) {
				CircularProgressIndicator(
					modifier = Modifier.size(24.dp),
					strokeWidth = 2.dp,
				)
			}
			is TrialEligibility.Resolved -> ProductList(products, trialEligibility.eligible, onSelect)
		}

		OutlineStyledButton(
			modifier = Modifier.fillMaxWidth().height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
			onClick = onDismiss,
			content = {
				Text(
					stringResource(R.string.cancel),
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
			},
		)
	}
}

// Row without a trial line: 2 × 24dp padding + one titleMedium line.
private val PRODUCT_ROW_HEIGHT = 72.dp

@Composable
private fun ProductList(products: List<ProductData>, trialEligible: Boolean, onSelect: (ProductData) -> Unit) {
	Column(
		modifier = Modifier
			.clip(RoundedCornerShape(8.dp))
			.fillMaxWidth()
			.border(
				BorderStroke(1.dp, MaterialTheme.colorScheme.primary),
				RoundedCornerShape(8.dp),
			)
			.background(MaterialTheme.colorScheme.primaryContainer),
	) {
		products.forEachIndexed { index, product ->
			Column(
				modifier = Modifier
					.fillMaxWidth()
					.clickable { onSelect(product) }
					.padding(vertical = 24.dp, horizontal = 16.dp),
				verticalArrangement = Arrangement.spacedBy(4.dp),
			) {
				Text(
					text = product.name + " (${product.price})",
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
				val trialDays = product.freeTrialDays
				if (trialEligible && trialDays != null) {
					Text(
						text = pluralStringResource(R.plurals.select_plan_modal_free_trial_days, trialDays, trialDays),
						style = MaterialTheme.typography.bodyLarge,
						color = MaterialTheme.colorScheme.onBackground,
					)
				}
			}

			if (index < products.lastIndex) {
				HorizontalDivider(
					color = MaterialTheme.colorScheme.primary.copy(alpha = 0.5f),
					thickness = 1.dp,
				)
			}
		}
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
			override val priceAmountMicros = null
			override val priceCurrencyCode = null
		},
		object : ProductData {
			override val id = "2"
			override val name = "Yearly Plan"
			override val price = "$49.99 / year"
			override val freeTrialDays = 7
			override val priceAmountMicros = null
			override val priceCurrencyCode = null
		},
	)

	NymVPNTheme(Theme.default()) {
		SubscriptionBottomSheetContent(
			products = mockProducts,
			trialEligibility = TrialEligibility.Resolved(eligible = true),
			onDismiss = {},
			onSelect = {},
		)
	}
}
