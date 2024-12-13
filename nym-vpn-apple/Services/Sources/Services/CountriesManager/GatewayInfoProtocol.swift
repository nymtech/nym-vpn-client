protocol GatewayInfoProtocol {
    var name: String { get }
    var countryCode: String? { get }
    var isGateway: Bool { get }
}
