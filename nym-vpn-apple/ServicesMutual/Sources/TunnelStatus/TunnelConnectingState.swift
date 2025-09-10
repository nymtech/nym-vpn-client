public enum TunnelConnectingState: Int, Codable {
    case resolvingApiAddresses
    case awaitingAccountReadiness
    case refreshingGateways
    case selectingGateways
    case connectingMixnetClient
    case connectingTunnel
    case unrecognized

    public var localizedStringKey: String {
        "tunnelConnectingState.\(self)"
    }
}
