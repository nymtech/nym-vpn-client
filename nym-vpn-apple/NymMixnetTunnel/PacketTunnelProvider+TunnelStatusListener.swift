import Foundation
import MixnetLibrary

extension PacketTunnelProvider: TunnelStatusListener {
    func onEvent(event: MixnetLibrary.TunnelEvent) {
        switch event {
        case let .newState(tunnelState):
            updateTunnelState(with: tunnelState)
        default:
            break
        }
    }

    func onBandwidthStatusChange(status: BandwidthStatus) {
        // todo: implement
    }

    func onConnectionStatusChange(status: ConnectionStatus) {
        // todo: implement
    }

    func onNymVpnStatusChange(status: NymVpnStatus) {
        // todo: implement
    }

    func onExitStatusChange(status: ExitStatus) {
        switch status {
        case .failure(let error):
            logger.error("onExitStatus: \(error.localizedDescription) after: \(connectionDuration())")
            scheduleDisconnectNotification()
        case .stopped:
            logger.info("onExitStatus: Tunnel stopped after: \(connectionDuration())")
        }
    }

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

    func updateTunnelState(with tunnelState: TunnelState) {
        switch tunnelState {
        case let .error(errorStateReason):
            updateLastError(with: errorStateReason)
        default:
            break
        }
    }

    func updateLastError(with errorStateReason: ErrorStateReason) {
        logger.error("onEvent: \(errorStateReason)")
        lastErrorStateReason = errorStateReason
    }
}
