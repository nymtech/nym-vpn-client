import Foundation
import NymVPNLib

extension PacketTunnelProvider: TunnelStatusListener {
    func onEvent(event: NymVPNLib.TunnelEvent) {
        tunnelActor.onEvent(event)
    }
}
