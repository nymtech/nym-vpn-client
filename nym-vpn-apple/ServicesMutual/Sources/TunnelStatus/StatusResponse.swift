public struct TunnelStatusResponse: Codable {
    public let status: TunnelStatus
    public let retryAttempt: Int?

    public init(status: TunnelStatus, retryAttempt: Int?) {
        self.status = status
        self.retryAttempt = retryAttempt
    }
}
