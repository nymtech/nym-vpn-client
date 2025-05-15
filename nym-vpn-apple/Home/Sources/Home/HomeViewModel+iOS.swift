#if os(iOS)
import Foundation
import Combine
import Tunnels
import UIComponents

extension HomeViewModel {
    func setupNetworkMonitorObservers() {
        // We use networkMonitor only as a source of truth for iOS disconnected state.
        // For macOS - we rely on daemon tunnel states.
        networkMonitor.$isAvailable
            .removeDuplicates()
            .debounce(for: .seconds(0.3), scheduler: DispatchQueue.global(qos: .background))
            .sink { [weak self] isAvailable in
                self?.offlineState(with: isAvailable)
            }
            .store(in: &cancellables)
    }

    func setupConnectionErrorObservers() {
        connectionManager.$lastError
            .receive(on: DispatchQueue.main)
            .sink { [weak self] error in
                MainActor.assumeIsolated {
                    self?.updateLastError(error)
                }
            }
            .store(in: &cancellables)
    }

    func configureTunnelStatusObservation(with tunnel: Tunnel) {
        tunnelStatusUpdateCancellable = tunnel.$status
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] status in
                MainActor.assumeIsolated {
                    self?.updateUI(with: status)
                }
            }

        tunnelRetryAttemptCancellable = tunnel.$retryAttempt
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] attempt in
                MainActor.assumeIsolated {
                    guard let self else { return }
                    self.updateStatusInfoState(
                        with: StatusInfoState(
                            tunnelStatus: tunnel.status,
                            isOnline: self.networkMonitor.isAvailable,
                            retryAttempt: attempt
                        )
                    )
                }
            }
    }
}
#endif
