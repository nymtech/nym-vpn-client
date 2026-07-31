import Combine
import Foundation
import TunnelStatus

/// True when launched in mock mode (compile-time MOCK_MODE, or a DEBUG launch arg).
public enum MockMode {
    public static var isEnabled: Bool {
        #if MOCK_MODE
        return true
        #elseif DEBUG
        return ProcessInfo.processInfo.arguments.contains("-MOCK_MODE")
            || ProcessInfo.processInfo.arguments.contains("MOCK_MODE")
        #else
        return false
        #endif
    }
}

/// Lightweight mock of connect/disconnect for UI testing (no daemon or network).
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
