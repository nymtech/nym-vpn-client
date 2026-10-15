package net.nymtech.nymvpn.ui.screens.server

import net.nymtech.nymvpn.util.extensions.isQuicSupported
import net.nymtech.nymvpn.util.extensions.scoreSorted
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import java.text.Collator
import java.util.Locale
import java.util.MissingResourceException

/**
 * Precomputed view of one gateway list: countries sorted for display, with display names,
 * lower-cased search terms, score-sorted gateways and optional region groups resolved once.
 * Built off the main thread whenever the underlying list changes; search and flattening then
 * only touch this structure.
 */
class GatewayIndex private constructor(
	val countries: List<CountryEntry>,
	/** Eligible gateways (after the QUIC filter), score-sorted, including ones without a country. */
	val gateways: List<NymGateway>,
	val countryCount: Int,
	val nodeCount: Int,
	val regionCountryCodes: Set<String>,
) {
	class CountryEntry(
		val code: String,
		val displayName: String,
		/** Lower-cased display name, ISO2 and ISO3 codes. */
		val searchTerms: List<String>,
		/** Lower-cased distinct regions of every gateway in the country. */
		val regionTerms: List<String>,
		val gateways: List<NymGateway>,
		/** Region groups, or null when the country is not displayed grouped by region. */
		val regions: List<RegionEntry>?,
		/** Gateways shown directly under the country header (all of them when [regions] is null). */
		val ungrouped: List<NymGateway>,
	) {
		val nodeCount: Int get() = gateways.size
	}

	class RegionEntry(val name: String, val gateways: List<NymGateway>)

	val countriesByCode: Map<String, CountryEntry> = countries.associateBy { it.code }

	companion object {
		val EMPTY = GatewayIndex(emptyList(), emptyList(), 0, 0, emptySet())

		fun build(gateways: List<NymGateway>, mode: Tunnel.Mode, quicOnly: Boolean, regionCountryCodes: Collection<String>, locale: Locale = Locale.getDefault()): GatewayIndex {
			val regionCodes = regionCountryCodes.map { it.lowercase() }.toSet()
			val eligible = gateways
				.filter { !quicOnly || it.isQuicSupported() }
				.distinctBy { it.identity }
				.scoreSorted(mode)
			val collator = Collator.getInstance(locale)
			val countries = eligible
				.filter { it.twoLetterCountryISO != null }
				.groupBy { it.twoLetterCountryISO!!.lowercase() }
				.map { (code, countryGateways) -> countryEntry(code, countryGateways, regionCodes, locale) }
				.sortedWith(compareBy(collator) { it.displayName })
			return GatewayIndex(
				countries = countries,
				gateways = eligible,
				countryCount = countries.size,
				nodeCount = eligible.size,
				regionCountryCodes = regionCodes,
			)
		}

		private fun countryEntry(code: String, gateways: List<NymGateway>, regionCodes: Set<String>, locale: Locale): CountryEntry {
			val javaLocale = Locale("", code)
			val displayName = javaLocale.getDisplayCountry(locale)
			val iso3 = try {
				javaLocale.isO3Country
			} catch (_: MissingResourceException) {
				""
			}
			val regionTerms = gateways.mapNotNull { it.region?.lowercase() }.distinct()
			val grouped = code in regionCodes && regionTerms.size >= 2
			val regions = if (grouped) {
				gateways.filter { it.region != null }
					.groupBy { it.region!! }
					.map { (name, regionGateways) -> RegionEntry(name, regionGateways) }
					.sortedBy { it.name }
			} else {
				null
			}
			return CountryEntry(
				code = code,
				displayName = displayName,
				searchTerms = listOf(displayName.lowercase(locale), code, iso3.lowercase()).filter { it.isNotEmpty() },
				regionTerms = regionTerms,
				gateways = gateways,
				regions = regions,
				ungrouped = if (grouped) gateways.filter { it.region == null } else gateways,
			)
		}
	}
}
