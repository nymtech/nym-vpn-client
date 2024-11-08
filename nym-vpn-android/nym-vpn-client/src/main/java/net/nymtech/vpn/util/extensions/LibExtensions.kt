package net.nymtech.vpn.util.extensions

import android.system.Os
import nym_vpn_lib.NetworkEnvironment

fun NetworkEnvironment.export() {
	Os.setenv("NETWORK_NAME", nymNetwork.networkName, true)

	Os.setenv("BECH32_PREFIX", nymNetwork.chainDetails.bech32AccountPrefix, true)
	Os.setenv("MIX_DENOM", nymNetwork.chainDetails.mixDenom.base, true)
	Os.setenv("MIX_DENOM_DISPLAY", nymNetwork.chainDetails.mixDenom.display, true)
	Os.setenv("STAKE_DENOM", nymNetwork.chainDetails.stakeDenom.base, true)
	Os.setenv("STAKE_DENOM_DISPLAY", nymNetwork.chainDetails.stakeDenom.display, true)
	Os.setenv("DENOMS_EXPONENT", nymNetwork.chainDetails.mixDenom.displayExponent.toString(), true)

	Os.setenv("MIXNET_CONTRACT_ADDRESS", nymNetwork.contracts.mixnetContractAddress, true)
	Os.setenv("VESTING_CONTRACT_ADDRESS", nymNetwork.contracts.vestingContractAddress, true)
	Os.setenv("GROUP_CONTRACT_ADDRESS", nymNetwork.contracts.groupContractAddress, true)
	Os.setenv("ECASH_CONTRACT_ADDRESS", nymNetwork.contracts.ecashContractAddress, true)
	Os.setenv("MULTISIG_CONTRACT_ADDRESS", nymNetwork.contracts.multisigContractAddress, true)
	Os.setenv("COCONUT_DKG_CONTRACT_ADDRESS", nymNetwork.contracts.coconutDkgContractAddress, true)

	nymNetwork.endpoints.firstOrNull()?.let {
		Os.setenv("NYXD", it.nyxdUrl, true)
		it.apiUrl?.let { url ->
			Os.setenv("NYM_API", url, true)
		}
		it.websocketUrl?.let { url ->
			Os.setenv("NYXD_WS", url, true)
		}
	}

	Os.setenv("NYM_VPN_API", nymVpnNetwork.nymVpnApiUrl, true)
}
