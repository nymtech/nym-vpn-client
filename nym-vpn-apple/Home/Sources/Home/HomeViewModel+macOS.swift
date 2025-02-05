#if os(macOS)
import Combine
import Foundation
import TunnelStatus

extension HomeViewModel {
    func setupGRPCManagerObservers() {
        grpcManager.$tunnelStatus
            .removeDuplicates()
            .receive(on: RunLoop.main)
            .sink { [weak self] status in
                self?.updateUI(with: status)
                self?.updateTimeConnected()
            }
            .store(in: &cancellables)
    }

    func setupDaemonStateObserver() {
        helperInstallManager.$daemonState.sink { [weak self] state in
            switch state {
            case .installing:
                self?.updateStatusInfoState(with: .installingDaemon)
                self?.updateConnectButtonState(with: .installingDaemon)
            case .installed:
                self?.updateStatusInfoState(with: .unknown)
                self?.updateConnectButtonState(with: .connect)
            case .unknown, .running:
                break
            }
        }
        .store(in: &cancellables)
    }

    func updateTimeConnected() {
        Task { @MainActor [weak self] in
            guard let self,
                  grpcManager.tunnelStatus == .connected,
                  let connectedDate = grpcManager.connectedDate
            else {
                self?.timeConnected = nil
                return
            }
            self.timeConnected = connectedDate
        }
    }
}
#endif
