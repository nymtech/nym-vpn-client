package net.nymtech.nymvpn.ui.screens.account.delete

enum class DeleteStep {
	EXPLAINER,
	PLAY_STOP,
	DELETED,
}

data class DeleteUiState(
	val isLoading: Boolean = true,
	val step: DeleteStep = DeleteStep.EXPLAINER,
	val showPlayRecords: Boolean = false,
	val isPlayStatusUnknown: Boolean = false,
	val timeLeftUntil: String? = null,
	val showConfirm: Boolean = false,
	val isDeleting: Boolean = false,
	val showError: Boolean = false,
)
