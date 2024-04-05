package net.nymtech.vpn.model

enum class VpnMode {
    FIVE_HOP_MIXNET,
    TWO_HOP_MIXNET;

    companion object {
        fun from(name: String?): VpnMode {
            return name?.let {
                try {
                    VpnMode.valueOf(it)
                } catch (e: IllegalArgumentException) {
                    default()
                }
            } ?: default()
        }

        fun default(): VpnMode {
            return TWO_HOP_MIXNET
        }
    }
}