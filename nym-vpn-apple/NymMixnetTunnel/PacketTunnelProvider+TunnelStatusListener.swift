import Foundation
import MixnetLibrary

extension PacketTunnelProvider: TunnelStatusListener {
    func onBandwidthStatusChange(status: BandwidthStatus) {
        // todo: implement
    }

    func onConnectionStatusChange(status: ConnectionStatus) {
        // todo: implement
    }

    func onNymVpnStatusChange(status: NymVpnStatus) {
        // todo: implement
    }

    func onExitStatusChange(status: ExitStatus) {}

    func onTunStatusChange(status: TunStatus) {
        eventContinuation.yield(.statusUpdate(status))
    }
}

private extension PacketTunnelProvider {
    func connectionDuration() -> String {
        var durationString = ""
        if let connectionStartDate {
            let dateFormatter = DateComponentsFormatter()
            dateFormatter.allowedUnits = [.hour, .minute, .second]
            dateFormatter.zeroFormattingBehavior = .pad
            durationString = dateFormatter.string(from: connectionStartDate, to: Date()) ?? ""
        }
        return durationString
    }
}
