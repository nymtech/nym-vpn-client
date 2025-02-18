#if os(macOS)
import Combine
import Constants
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

        grpcManager.$errorReason.sink { [weak self] error in
            self?.lastError = error
        }
        .store(in: &cancellables)

        grpcManager.$generalError.sink { [weak self] error in
            self?.lastError = error
            if error == GeneralNymError.noMnemonicStored {
                Task { @MainActor [weak self] in
                    self?.navigateToAddCredentials()
                }
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
