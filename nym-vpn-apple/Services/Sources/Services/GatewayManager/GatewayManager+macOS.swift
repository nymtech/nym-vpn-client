#if os(macOS)
import Foundation
import CountriesManagerTypes

extension GatewayManager {
    func setupDaemonObserver() {
        grpcManager.$isServing
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] isServing in
                guard isServing else { return }
                Task {
                    await self?.fetchGateways()
                }
            }
            .store(in: &cancellables)
    }
}
#endif
