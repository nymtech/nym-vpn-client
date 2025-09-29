package net.nymtech.nymvpn.ui.screens.account.plan.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedCard
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.model.ProductData
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SubscriptionBottomSheet(
	products: List<ProductData>,
	onDismiss: () -> Unit,
	onSelect: (ProductData) -> Unit
) {
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
fun SubscriptionBottomSheetContent(
	products: List<ProductData>,
	onDismiss: () -> Unit,
	onSelect: (ProductData) -> Unit
) {
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

		products.forEach { product ->
			OutlinedCard(
				onClick = { onSelect(product) },
				modifier = Modifier.fillMaxWidth(),
				shape = RoundedCornerShape(16.dp),
			) {
				Column(
					modifier = Modifier
						.fillMaxWidth()
						.padding(16.dp),
					verticalArrangement = Arrangement.spacedBy(4.dp),
				) {
					Text(
						text = product.name,
						style = MaterialTheme.typography.titleMedium,
						color = MaterialTheme.colorScheme.onSurface,
					)
					Text(
						text = product.price,
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onSurfaceVariant,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular))
					)
				}
			}
		}

		Spacer(Modifier.height(12.dp))

		TextButton(
			onClick = onDismiss,
			modifier = Modifier
				.align(Alignment.CenterHorizontally)
				.fillMaxWidth(),
		) {
			Text(
				text = stringResource(R.string.cancel),
				style = MaterialTheme.typography.titleMedium,
				color = MaterialTheme.colorScheme.primary,
			)
		}
	}
}

@Preview
@Composable
fun SubscriptionBottomSheetContentPreview() {
	val previewProducts = listOf(
		ProductData(id = "", name = "Basic Plan", price = "$4.99 / month"),
		ProductData(id = "",name = "Pro Plan", price = "$9.99 / month"),
		ProductData(id = "",name = "Premium Plan", price = "$14.99 / month"),
	)

	NymVPNTheme(Theme.default()) {
		SubscriptionBottomSheetContent(
			products = previewProducts,
			onDismiss = {},
			onSelect = {}
		)
	}
}
