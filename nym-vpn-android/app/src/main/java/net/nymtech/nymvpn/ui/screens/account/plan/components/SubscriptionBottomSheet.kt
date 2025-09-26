package net.nymtech.nymvpn.ui.screens.account.plan.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.model.ProductData

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SubscriptionBottomSheet(products: List<ProductData>, onDismiss: () -> Unit, onSelect: (ProductData) -> Unit) {
	ModalBottomSheet(
		onDismissRequest = onDismiss,
		shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
		containerColor = MaterialTheme.colorScheme.surface,
		tonalElevation = 4.dp,
	) {
		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(16.dp),
			verticalArrangement = Arrangement.spacedBy(12.dp),
		) {
			Text(
				text = "Choose your plan",
				style = MaterialTheme.typography.titleLarge,
				color = MaterialTheme.colorScheme.onSurface,
				modifier = Modifier.align(Alignment.CenterHorizontally),
			)

			Spacer(Modifier.height(8.dp))

			products.forEach { product ->
				ElevatedCard(
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
					text = "Cancel",
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.primary,
				)
			}
		}
	}
}
