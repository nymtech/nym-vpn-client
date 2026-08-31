public enum TunnelConnectingState: Int, Codable {
    case resolvingApiAddresses
    case awaitingAccountReadiness
    case awaitingCredentialsAvailability
    case refreshingGateways
    case selectingGateways
    case registeringWithGateways
    case connectingTunnel
    case unrecognized

    public var localizedStringKey: String {
        "tunnelConnectingState.\(self)"
    }
}
