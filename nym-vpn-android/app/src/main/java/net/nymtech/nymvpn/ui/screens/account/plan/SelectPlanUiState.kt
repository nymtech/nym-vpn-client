package net.nymtech.nymvpn.ui.screens.account.plan

import net.nymtech.billing.model.ProductData

data class SelectPlanUiState(val subscriptions: List<ProductData> = emptyList())
