#if os(iOS)
import NetworkExtension
import WidgetKit

@available(iOS 18.0, *)
public struct VPNControlStatusValueProvider: ControlValueProvider {
    public typealias Value = VPNStatus

    public var previewValue: VPNStatus {
        .status(.disconnected)
    }

    public init() {}

    public func currentValue() async throws -> VPNStatus {
        guard let manager = try await NymTunnelManager.loadManager()
        else {
            return .notConfigured
        }
        return .status(manager.connection.status)
    }
}
#endif
