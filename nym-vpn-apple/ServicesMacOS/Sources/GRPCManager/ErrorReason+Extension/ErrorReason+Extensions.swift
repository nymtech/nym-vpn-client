import GRPC
import ErrorReason

extension ErrorReason {
    init(with errorStateReason: Nym_Vpn_TunnelState.ErrorStateReason) {
        switch errorStateReason {
        case .firewall:
            self = .firewall
        case .routing:
            self = .routing
        case .dns:
            self = .dns
        case .tunDevice:
            self = .tunDevice
        case .tunnelProvider:
            self = .tunnelProvider
        case .internal:
            self = .internalUnknown
        case .sameEntryAndExitGateway:
            self = .sameEntryAndExitGateway
        case .invalidEntryGatewayCountry:
            self = .invalidEntryGatewayCountry
        case .invalidExitGatewayCountry:
            self = .invalidExitGatewayCountry
        case .badBandwidthIncrease:
            self = .badBandwidthIncrease
        case .duplicateTunFd:
            self = .duplicateTunFd
        case .UNRECOGNIZED:
            self = .unknown
        }
    }
}
