package net.nymtech.vpn.util

import kotlin.math.floor
import kotlin.math.ln
import kotlin.math.pow

// TODO this needs ipv6 support added
object AllowedIpsCalculator {

	private val CIDR2MASK: IntArray = intArrayOf(
		0x00000000, -0x80000000,
		-0x40000000, -0x20000000, -0x10000000, -0x8000000, -0x4000000,
		-0x2000000, -0x1000000, -0x800000, -0x400000, -0x200000,
		-0x100000, -0x80000, -0x40000, -0x20000, -0x10000,
		-0x8000, -0x4000, -0x2000, -0x1000, -0x800,
		-0x400, -0x200, -0x100, -0x80, -0x40,
		-0x20, -0x10, -0x8, -0x4, -0x2,
		-0x1,
	)

	private val rangeStart = mutableListOf<Long>()
	private val rangeEnd = mutableListOf<Long>()

	// Custom comparator for sorting networks based on IP address only

	private fun Long.toIpString(): String {
		return "${(this shr 24) and 0xFF}.${(this shr 16) and 0xFF}.${(this shr 8) and 0xFF}.${this and 0xFF}"
	}

	private fun sortRanges() {
		// Remove elements with value -1 and maintain the relationship with 're'
		val filteredPairs = rangeStart.mapIndexedNotNull { index, value ->
			if (value != -1L) Pair(value, rangeEnd[index]) else null
		}

		// Separate the filtered values back into two lists
		val filteredRs = filteredPairs.map { it.first }.toMutableList()
		val filteredRe = filteredPairs.map { it.second }.toMutableList()

		// Sort the lists based on 'rs'
		val sortedIndices = filteredRs.indices.sortedBy { filteredRs[it] }

		// Create new sorted lists
		val sortedRs = sortedIndices.map { filteredRs[it] }.toMutableList()
		val sortedRe = sortedIndices.map { filteredRe[it] }.toMutableList()

		// Clear original lists and add sorted values
		rangeStart.clear()
		rangeEnd.clear()
		rangeStart.addAll(sortedRs)
		rangeEnd.addAll(sortedRe)
	}

	private fun mergeRanges() {
		val size = rangeStart.size

		for (i in 0 until size - 1) {
			val j = i + 1
			// Check if the end of the current range is adjacent to the start of the next range
			if (rangeEnd[i] == rangeStart[j] - 1) {
				// Merge ranges by updating the second range's start and marking the first as invalid
				rangeStart[j] = rangeStart[i]
				rangeStart[i] = -1
				rangeEnd[i] = -1
			}
		}
		// After merging, sort the ranges to remove invalid entries
		sortRanges()
	}

	private fun removeRange(ip1: Long, ip2: Long) {
		val size = rangeStart.size

		for (i in 0 until size) {
			if (rangeStart[i] > ip2 || rangeEnd[i] < ip1) continue
			if (ip1 <= rangeStart[i] && ip2 >= rangeEnd[i]) {
				rangeStart[i] = -1
				rangeEnd[i] = -1
			} else if (ip1 <= rangeStart[i]) {
				rangeStart[i] = ip2 + 1
			} else if (ip2 >= rangeEnd[i]) {
				rangeEnd[i] = ip1 - 1
			} else {
				rangeStart.add(ip2 + 1)
				rangeEnd.add(rangeEnd[i])
				rangeEnd[i] = ip1 - 1
			}
		}
		sortRanges()
	}

	private fun addRange(ip1: Long, ip2: Long) {
		// First, remove any overlapping ranges
		removeRange(ip1, ip2)

		// Add the new range
		rangeStart.size
		rangeStart.add(ip1)
		rangeEnd.add(ip2)

		// Sort and merge ranges after addition
		sortRanges()
		mergeRanges()
	}

	private fun cidrToRange(ip: String, width: Int): Pair<Long, Long> {
		val ipNum = ip.ipToLong()
		val mask = (0xFFFFFFFF shl (32 - width)) and 0xFFFFFFFF
		val ip1 = ipNum and mask
		val ip2 = ip1 + (0xFFFFFFFF shr width)
		return Pair(ip1, ip2)
	}

	fun calculateAllowedIps(addIps: List<String>, removeIps: List<String>): List<String> {
		if (removeIps.isEmpty()) return addIps
		addIps.forEach {
			when {
				it.contains("/") -> {
					val parts = it.split("/")
					val ip = parts[0]
					val width = parts[1].toInt()
					val (ip1, ip2) = cidrToRange(ip, width)
					addRange(ip1, ip2)
				}
				else -> {
					val ip = it
					val ipNum = ip.ipToLong()
					addRange(ipNum, ipNum)
				}
			}
		}

		removeIps.forEach {
			when {
				it.contains("/") -> {
					val parts = it.split("/")
					val ip = parts[0]
					val width = parts[1].toInt()
					val (ip1, ip2) = cidrToRange(ip, width)
					removeRange(ip1, ip2)
				}
				else -> {
					val ip = it
					val ipNum = ip.ipToLong()
					removeRange(ipNum, ipNum)
				}
			}
		}

		return buildAllowedIpsList()
	}

	private fun buildAllowedIpsList(): List<String> {
		return rangeStart.mapIndexed { i, ip ->
			rangeToCIDRList(ip.toIpString(), rangeEnd[i].toIpString())
		}.flatten()
	}

	private fun rangeToCIDRList(startIp: String, endIp: String): List<String> {
		var start = startIp.ipToLong()
		val end = endIp.ipToLong()

		val pairs = ArrayList<String>()
		while (end >= start) {
			var maxsize: Byte = 32
			while (maxsize > 0) {
				val mask = CIDR2MASK[maxsize - 1].toLong()
				val maskedBase = start and mask

				if (maskedBase != start) {
					break
				}

				maxsize--
			}
			val x = ln((end - start + 1).toDouble()) / ln(2.0)
			val maxDiff = (32 - floor(x)).toInt().toByte()
			if (maxsize < maxDiff) {
				maxsize = maxDiff
			}
			val ip = (start).toIpString()
			pairs.add("$ip/$maxsize")
			start += 2.0.pow(((32 - maxsize).toDouble())).toLong()
		}
		return pairs
	}

	private fun String.ipToLong(): Long {
		val octets = this.split(".")
		return (octets[0].toLong() shl 24) or
			(octets[1].toLong() shl 16) or
			(octets[2].toLong() shl 8) or
			(octets[3].toLong())
	}
}
