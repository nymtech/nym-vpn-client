package net.nymtech.nymvpn.ui.screens.onboarding

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import javax.inject.Inject

@HiltViewModel
class OnboardingViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	fun onContinueClicked() = viewModelScope.launch {
		settingsRepository.setOnboardingCompleted()
	}
}
