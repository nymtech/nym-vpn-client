package net.nymtech.vpnclient.tunnel

enum class ParameterGenerationError {
	NoMatchingRelay,
	NoMatchingBridgeRelay,
	NoWireguardKey,
	CustomTunnelHostResultionError,
}
