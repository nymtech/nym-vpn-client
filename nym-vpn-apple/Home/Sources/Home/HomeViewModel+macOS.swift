#if os(macOS)
import Combine
import Constants
import Foundation
import TunnelStatus

extension HomeViewModel {
    func setupGRPCManagerObservers() {
        grpcManager.$tunnelStatus
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .sink { status in
                MainActor.assumeIsolated {
                    self.updateUI(with: status)
                }
            }
            .store(in: &cancellables)

        grpcManager.$errorReason
            .dropFirst()
            .receive(on: DispatchQueue.main)
            .sink { error in
                MainActor.assumeIsolated {
                    self.updateLastError(error)
                }
            }
            .store(in: &cancellables)
    }
}
#endif
