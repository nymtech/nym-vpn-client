#if os(macOS)
import Combine
import Constants
import Foundation
import TunnelStatus
import UIComponents

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

        grpcManager.$tunnelConnectingState
            .receive(on: DispatchQueue.main)
            .sink { state in
                MainActor.assumeIsolated {
                    guard self.lastTunnelStatus == .connecting else { return }
                    self.updateStatusInfoState(
                        with: StatusInfoState(
                            tunnelStatus: self.lastTunnelStatus,
                            isOnline: self.networkMonitor.isAvailable,
                            retryAttempt: self.connectionManager.connectionRetryAttempt,
                            tunnelConnectingState: state
                        )
                    )
                }
            }
            .store(in: &cancellables)
    }
}
#endif
