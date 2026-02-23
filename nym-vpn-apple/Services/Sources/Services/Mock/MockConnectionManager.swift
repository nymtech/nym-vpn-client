import Combine
import Foundation
import TunnelStatus

/// Detects whether the app was launched in mock mode.
/// Checks both environment variable and launch argument.
public enum MockMode {
    public static var isEnabled: Bool {
        #if MOCK_MODE
        return true
        #else
        return ProcessInfo.processInfo.environment["MOCK_MODE"] == "1"
            || ProcessInfo.processInfo.arguments.contains("-MOCK_MODE")
        #endif
    }
}

/// Lightweight mock of VPN connection behavior for UI testing.
/// Simulates connect/disconnect state transitions with delays,
/// without requiring the real nym-vpnd daemon or network.
@MainActor
public final class MockConnectionState: ObservableObject {
    @Published public var tunnelStatus: TunnelStatus = .disconnected
    @Published public var isAccountStored: Bool = true

    public static let shared = MockConnectionState()

    private init() {}

    public func connect() {
        tunnelStatus = .connecting
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 1_500_000_000)
            tunnelStatus = .connected
        }
    }

    public func disconnect() {
        tunnelStatus = .disconnecting
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 800_000_000)
            tunnelStatus = .disconnected
        }
    }
}
