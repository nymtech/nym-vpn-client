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
    /// Absent when sent by an extension older than this field. Prefer it over `status` and `lastError`.
    public let state: TunnelStateResponse?

    public init(
        status: TunnelStatus,
        retryAttempt: Int?,
        afterDisconnectAction: AfterDisconnectAction?,
        lastError: ErrorReason?,
        tunnelConnectingState: TunnelConnectingState?,
        connectionInfoData: ConnectionInfoData?,
        connectedAt: Date? = nil,
        state: TunnelStateResponse? = nil
    ) {
        self.status = status
        self.retryAttempt = retryAttempt
        self.afterDisconnectAction = afterDisconnectAction
        self.lastError = lastError
        self.tunnelConnectingState = tunnelConnectingState
        self.connectionInfoData = connectionInfoData
        self.connectedAt = connectedAt
        self.state = state
    }
}
