package net.nymtech.nymvpn.service.country

interface CountryCacheService {
	suspend fun updateExitCountriesCache(): Result<Unit>

	suspend fun updateEntryCountriesCache(): Result<Unit>

	suspend fun updateLowLatencyEntryCountryCache(): Result<Unit>
}
