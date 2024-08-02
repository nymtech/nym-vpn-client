package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import androidx.compose.runtime.mutableStateListOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.FileUtils
import net.nymtech.vpn.model.License
import javax.inject.Inject

@HiltViewModel
class LicensesViewModel
@Inject
constructor(
	private val fileUtils: FileUtils,
) : ViewModel() {
	private val _licences = mutableStateListOf<Artifact>()
	val licenses: List<Artifact>
		get() = _licences

	fun loadLicensesFromAssets() = viewModelScope.launch {
		val kotlinLicenseJson = fileUtils.readTextFromAssetsFile(Constants.KOTLIN_LICENSES_ASSET_FILE_NAME)
		val artifacts = kotlinLicenseJson.getOrNull()?.let {
			Artifact.fromJsonList(it).getOrNull() ?: emptyList()
		} ?: emptyList()
		val rustLicenseJson = fileUtils.readTextFromAssetsFile(Constants.RUST_LICENSES_ASSET_FILE_NAME)
		val rustLicenses = rustLicenseJson.getOrNull()?.let {
			License.fromJsonList(it).getOrNull() ?: emptyList()
		} ?: emptyList()
		_licences.addAll(
			Artifact.from(rustLicenses) + artifacts,
		)
	}
}
