package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.util.FileUtils
import javax.inject.Inject

@HiltViewModel
class LicensesViewModel
@Inject
constructor(
	private val fileUtils: FileUtils,
) : ViewModel() {
	private val _licences = MutableStateFlow<List<Artifact>>(emptyList())
	val licenses = _licences.asStateFlow()
	private val licensesFileName = "artifacts.json"

	fun loadLicensesFromAssets() = viewModelScope.launch {
		val text = fileUtils.readTextFromFileName(licensesFileName)
		_licences.update {
			LicenseParser.decode(text)
		}
	}
}
