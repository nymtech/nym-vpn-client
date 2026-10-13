package net.nymtech.billing.model

sealed interface PurchaseFlowResult {
	/** Outcome, including errors, arrives via `uiState`. */
	data object Started : PurchaseFlowResult

	/** Nothing reached `uiState`; the caller must surface it. */
	data class Failed(val reason: String) : PurchaseFlowResult
}
