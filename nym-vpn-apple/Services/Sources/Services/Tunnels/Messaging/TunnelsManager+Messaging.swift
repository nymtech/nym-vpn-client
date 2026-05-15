#if os(iOS)
import Foundation
import NetworkExtension

extension TunnelsManager {
    /// Sends a setter command to the running tunnel extension.
    /// Persistence of the new value lives at the call site — this only delivers the live update.
    public func send(_ message: TunnelProviderMessage) async {
        guard let activeTunnel,
              activeTunnel.status == .connected || activeTunnel.status == .connecting
        else {
            return
        }
        do {
            let data = try message.encode()
            _ = try await activeTunnel.sendProviderMessage(with: data)
        } catch {
            logger.error("Failed to send tunnel message: \(error.localizedDescription)")
        }
    }
}
#endif
