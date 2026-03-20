package net.nymtech.nymvpn.ui.screens.account.plan

import net.nymtech.billing.model.ProductData
import net.nymtech.nymvpn.ui.screens.account.info.AutologinState

data class SelectPlanUiState(val subscriptions: List<ProductData> = emptyList(), val autologin: AutologinState = AutologinState.Idle)
