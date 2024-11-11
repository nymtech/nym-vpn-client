import Foundation
import MixnetLibrary

extension PacketTunnelProvider: TunnelStatusListener {
    func onEvent(event: MixnetLibrary.TunnelEvent) {
        tunnelActor.onEvent(event)
    }
}
