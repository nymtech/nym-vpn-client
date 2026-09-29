import Foundation
import ErrorReason

public struct TunnelStatusResponse: Codable {
    public let status: TunnelStatus
    public let retryAttempt: Int?
    public let afterDisconnectAction: AfterDisconnectAction?
    public let lastError: ErrorReason?
    public let tunnelConnectingState: TunnelConnectingState?
    public let connectionInfoData: ConnectionInfoData?
    public let connectedAt: Date?

    public init(
        status: TunnelStatus,
        retryAttempt: Int?,
        afterDisconnectAction: AfterDisconnectAction?,
        lastError: ErrorReason?,
        tunnelConnectingState: TunnelConnectingState?,
        connectionInfoData: ConnectionInfoData?,
        connectedAt: Date? = nil
    ) {
        self.status = status
        self.retryAttempt = retryAttempt
        self.afterDisconnectAction = afterDisconnectAction
        self.lastError = lastError
        self.tunnelConnectingState = tunnelConnectingState
        self.connectionInfoData = connectionInfoData
        self.connectedAt = connectedAt
    }
}
