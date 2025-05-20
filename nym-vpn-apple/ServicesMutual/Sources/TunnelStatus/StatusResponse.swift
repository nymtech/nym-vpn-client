public struct TunnelStatusResponse: Codable {
    public let status: TunnelStatus
    public let retryAttempt: Int?
    public let afterDisconnectAction: AfterDisconnectAction?

    public init(status: TunnelStatus, retryAttempt: Int?, afterDisconnectAction: AfterDisconnectAction?) {
        self.status = status
        self.retryAttempt = retryAttempt
        self.afterDisconnectAction = afterDisconnectAction
    }
}
