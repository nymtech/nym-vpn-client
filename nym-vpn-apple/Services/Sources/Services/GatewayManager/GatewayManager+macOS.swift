#if os(macOS)
import Foundation
import ConnectionTypes

extension GatewayManager {
    func setupDaemonObserver() {
        grpcManager.$isServing
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] isServing in
                guard isServing else { return }
                Task {
                    self?.updateGateways()
                }
            }
            .store(in: &cancellables)
    }
}
#endif
