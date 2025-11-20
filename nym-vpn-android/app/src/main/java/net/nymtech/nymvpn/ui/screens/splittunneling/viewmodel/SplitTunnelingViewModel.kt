package net.nymtech.nymvpn.ui.screens.splittunneling.viewmodel

import android.Manifest
import android.content.Context
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.SplitTunnelingRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.ui.screens.splittunneling.model.AppFilter
import net.nymtech.nymvpn.ui.screens.splittunneling.model.AppInfo
import net.nymtech.nymvpn.ui.screens.splittunneling.model.SplitTunnelingUiState
import net.nymtech.nymvpn.ui.screens.splittunneling.model.UiEvent
import net.nymtech.vpn.backend.Tunnel
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class SplitTunnelingViewModel @Inject constructor(
	@param:ApplicationContext private val context: Context,
	private val splitTunnelingRepository: SplitTunnelingRepository,
	private val backendManager: BackendManager,
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	private val packageManager = context.packageManager
	private val _uiState = MutableStateFlow(SplitTunnelingUiState())
	val uiState = _uiState.asStateFlow()
	private var initialAppInfoList: List<AppInfo> = emptyList()

	private val applicationFilterPredicate: (ApplicationInfo) -> Boolean = { appInfo ->
		hasInternetPermission(appInfo.packageName) && !isSelfApplication(appInfo.packageName)
	}

	init {
		getAllInstalledAppList()
		onPerAppSecurityBannerDisplayed()
	}

	fun onUiEvent(event: UiEvent) {
		when (event) {
			is UiEvent.QueryChange -> filterApps(event.query)
			UiEvent.SelectAllDirectAppsClick -> filterAllDirectApps()
			UiEvent.SelectAllVpnPassThroughClick -> filterAllVpnPassThroughApps()
			is UiEvent.ChangeSelection -> changeChoiceSelection(event.packageName)
			is UiEvent.OnBackClick -> onBackClick(event.tunnelState)
			is UiEvent.ClearNavigation -> _uiState.update { it.copy(pendingNavigation = null) }
			is UiEvent.ClearDialog -> _uiState.update { it.copy(pendingDialog = null) }
			is UiEvent.NavigateBack -> _uiState.update { it.copy(pendingNavigation = SplitTunnelingUiState.PendingNavigation.NavigateBack) }
		}
	}

	fun disconnect() {
		viewModelScope.launch {
			backendManager.stopTunnel()
			_uiState.update { it.copy(pendingDialog = null, pendingNavigation = SplitTunnelingUiState.PendingNavigation.NavigateToHome) }
		}
	}

	private fun onPerAppSecurityBannerDisplayed() = viewModelScope.launch {
		settingsRepository.setIsStreamServerBannerDisplayed(true)
	}

	private fun getAllInstalledAppList() {
		viewModelScope.launch {
			runCatching {
				val savedAppsInfo = withContext(Dispatchers.IO) {
					splitTunnelingRepository.getAppInfoList().associateBy { it.packageName }
				}

				val normalApps = mutableListOf<AppInfo>()
				val systemApps = mutableListOf<AppInfo>()

				val installedApps = packageManager.getInstalledApplications(PackageManager.GET_META_DATA)
					.filter(applicationFilterPredicate)
					.distinctBy {
						it.packageName
					}

				for (appInfo in installedApps) {
					val name = appInfo.loadLabel(packageManager).toString()
					val icon = appInfo.icon

					val app = AppInfo(
						name = name,
						packageName = appInfo.packageName,
						icon = icon,
						passThroughVpn = savedAppsInfo[appInfo.packageName]?.passThroughVpn ?: true,
					)

					if (appInfo.flags and ApplicationInfo.FLAG_SYSTEM != 0 || appInfo.flags and ApplicationInfo.FLAG_UPDATED_SYSTEM_APP != 0) {
						if (isLaunchable(appInfo.packageName)) systemApps.add(app)
					} else {
						normalApps.add(app)
					}
				}

				withContext(Dispatchers.IO) {
					splitTunnelingRepository.saveAppInfoList(systemApps + normalApps)
				}

				val sortedSystemApps = systemApps.sortedBy { app -> app.name }
				val sortedNormalApps = normalApps.sortedBy { app -> app.name }

				initialAppInfoList = sortedSystemApps + sortedNormalApps

				_uiState.update {
					it.copy(
						systemApps = sortedSystemApps,
						normalApps = sortedNormalApps,
						filteredSystemApps = sortedSystemApps,
						filteredNormalApps = sortedNormalApps,
						directAppsCount = sortedSystemApps.totalAppCounts(false) + sortedNormalApps.totalAppCounts(false),
						vpnPassThroughAppsCount = sortedSystemApps.totalAppCounts(true) + sortedNormalApps.totalAppCounts(true),
					)
				}
			}.onFailure {
				Timber.e("error in getAllInstalledAppList: $it")
			}
		}
	}

	private fun filterApps(query: String) {
		viewModelScope.launch {
			_uiState.update {
				val queryFilteredSystemApps = it.systemApps.filter { app -> app.name.contains(query, ignoreCase = true) }
				val queryFilteredNormalApps = it.normalApps.filter { app -> app.name.contains(query, ignoreCase = true) }
				val filteredSystemApps = queryFilteredSystemApps.filter { app -> it.appliedFilter == AppFilter.None || app.passThroughVpn == (it.appliedFilter == AppFilter.VpnPassThrough) }
				val filteredNormalApps = queryFilteredNormalApps.filter { app -> it.appliedFilter == AppFilter.None || app.passThroughVpn == (it.appliedFilter == AppFilter.VpnPassThrough) }
				val directAppsCount = if (it.appliedFilter == AppFilter.VpnPassThrough) {
					queryFilteredSystemApps.totalAppCounts(false) + queryFilteredNormalApps.totalAppCounts(false)
				} else {
					filteredSystemApps.totalAppCounts(false) + filteredNormalApps.totalAppCounts(false)
				}
				val vpnPassThroughAppsCount = if (it.appliedFilter == AppFilter.Direct) {
					queryFilteredSystemApps.totalAppCounts(true) + queryFilteredNormalApps.totalAppCounts(true)
				} else {
					filteredSystemApps.totalAppCounts(true) + filteredNormalApps.totalAppCounts(true)
				}

				it.copy(
					query = query,
					filteredSystemApps = filteredSystemApps,
					filteredNormalApps = filteredNormalApps,
					directAppsCount = directAppsCount,
					vpnPassThroughAppsCount = vpnPassThroughAppsCount,
				)
			}
		}
	}

	private fun filterAllDirectApps() {
		viewModelScope.launch {
			_uiState.update {
				val isAlreadySelected = it.appliedFilter == AppFilter.Direct
				val filteredSystemApps = if (isAlreadySelected) it.systemApps else it.systemApps.filterAllPassThroughValue(false)
				val filteredNormalApps = if (isAlreadySelected) it.normalApps else it.normalApps.filterAllPassThroughValue(false)
				it.copy(
					filteredSystemApps = filteredSystemApps,
					filteredNormalApps = filteredNormalApps,
					directAppsCount = filteredSystemApps.totalAppCounts(false) + filteredNormalApps.totalAppCounts(false),
					vpnPassThroughAppsCount = it.systemApps.totalAppCounts(true) + it.normalApps.totalAppCounts(true),
					appliedFilter = if (!isAlreadySelected) AppFilter.Direct else AppFilter.None,
				)
			}
			if (uiState.value.query.isNotEmpty()) {
				filterApps(uiState.value.query)
			}
		}
	}

	private fun filterAllVpnPassThroughApps() {
		viewModelScope.launch {
			_uiState.update {
				val isAlreadySelected = it.appliedFilter == AppFilter.VpnPassThrough
				val filteredSystemApps = if (isAlreadySelected) it.systemApps else it.systemApps.filterAllPassThroughValue(true)
				val filteredNormalApps = if (isAlreadySelected) it.normalApps else it.normalApps.filterAllPassThroughValue(true)
				it.copy(
					filteredSystemApps = filteredSystemApps,
					filteredNormalApps = filteredNormalApps,
					directAppsCount = it.systemApps.totalAppCounts(false) + it.normalApps.totalAppCounts(false),
					vpnPassThroughAppsCount = filteredSystemApps.totalAppCounts(true) + filteredNormalApps.totalAppCounts(true),
					appliedFilter = if (!isAlreadySelected) AppFilter.VpnPassThrough else AppFilter.None,
				)
			}
			if (uiState.value.query.isNotEmpty()) {
				filterApps(uiState.value.query)
			}
		}
	}

	private fun changeChoiceSelection(packageName: String) {
		viewModelScope.launch {
			val updatedSystemApps = _uiState.value.systemApps.updatePassThroughValue(packageName)
			val updatedNormalApps = _uiState.value.normalApps.updatePassThroughValue(packageName)

			withContext(Dispatchers.IO) {
				splitTunnelingRepository.saveAppInfoList(updatedSystemApps + updatedNormalApps)
			}

			_uiState.update {
				val filteredSystemApps = it.filteredSystemApps
					.updatePassThroughValue(packageName)
					.filter { app -> it.appliedFilter == AppFilter.None || app.passThroughVpn == (it.appliedFilter == AppFilter.VpnPassThrough) }
				val filteredNormalApps = it.filteredNormalApps
					.updatePassThroughValue(packageName)
					.filter { app -> it.appliedFilter == AppFilter.None || app.passThroughVpn == (it.appliedFilter == AppFilter.VpnPassThrough) }
				val directAppsCount = if (it.appliedFilter == AppFilter.VpnPassThrough) {
					updatedSystemApps.totalAppCounts(false) + updatedNormalApps.totalAppCounts(false)
				} else {
					filteredSystemApps.totalAppCounts(false) + filteredNormalApps.totalAppCounts(false)
				}
				val vpnPassThroughAppsCount = if (it.appliedFilter == AppFilter.Direct) {
					updatedSystemApps.totalAppCounts(true) + updatedNormalApps.totalAppCounts(true)
				} else {
					filteredSystemApps.totalAppCounts(true) + filteredNormalApps.totalAppCounts(true)
				}

				it.copy(
					systemApps = updatedSystemApps,
					normalApps = updatedNormalApps,
					filteredSystemApps = filteredSystemApps,
					filteredNormalApps = filteredNormalApps,
					directAppsCount = directAppsCount,
					vpnPassThroughAppsCount = vpnPassThroughAppsCount,
				)
			}
		}
	}

	private fun onBackClick(tunnelState: Tunnel.State) {
		if (tunnelState != Tunnel.State.Up) {
			_uiState.update { it.copy(pendingNavigation = SplitTunnelingUiState.PendingNavigation.NavigateBack) }
			Timber.d("onBackClick: NavigateBack ${_uiState.value}")
		} else {
			if (initialAppInfoList != uiState.value.systemApps + uiState.value.normalApps) {
				_uiState.update { it.copy(pendingDialog = SplitTunnelingUiState.PendingDialog.AppListChangeDialog) }
			} else {
				_uiState.update { it.copy(pendingNavigation = SplitTunnelingUiState.PendingNavigation.NavigateBack) }
			}
		}
	}

	private fun List<AppInfo>.updatePassThroughValue(packageName: String) =
		map { appInfo -> if (appInfo.packageName == packageName) appInfo.copy(passThroughVpn = !appInfo.passThroughVpn) else appInfo }

	private fun List<AppInfo>.filterAllPassThroughValue(passThroughVpn: Boolean) = filter { appInfo -> appInfo.passThroughVpn == passThroughVpn }

	private fun List<AppInfo>.totalAppCounts(passThroughVpn: Boolean) = filter { app -> app.passThroughVpn == passThroughVpn }.size

	private fun hasInternetPermission(packageName: String): Boolean {
		return PackageManager.PERMISSION_GRANTED ==
			packageManager.checkPermission(Manifest.permission.INTERNET, packageName)
	}

	private fun isLaunchable(packageName: String): Boolean {
		return packageManager.getLaunchIntentForPackage(packageName) != null ||
			packageManager.getLeanbackLaunchIntentForPackage(packageName) != null
	}

	private fun isSelfApplication(packageName: String): Boolean {
		return packageName == BuildConfig.APPLICATION_ID
	}
}
