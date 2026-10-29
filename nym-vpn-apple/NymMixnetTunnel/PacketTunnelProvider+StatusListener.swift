import Foundation
import NymVPNLib

extension PacketTunnelProvider: StatusListener {
    func onEvent(event: NymVPNLib.Event) {
        tunnelActor.onEvent(event)
    }
}
