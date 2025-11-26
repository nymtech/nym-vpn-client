import Foundation
import Combine
import ConfigurationManager
#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif

@MainActor public final class FeatureFlagsManager: ObservableObject {
#if os(macOS)
    private let grpcManager: GRPCManager
#endif
    private let configurationManager: ConfigurationManager

    private var cancellables = Set<AnyCancellable>()

#if os(iOS)
    public static let shared = FeatureFlagsManager(configurationManager: .shared)
#elseif os(macOS)
    public static let shared = FeatureFlagsManager(
        configurationManager: .shared,
        grpcManager: .shared
    )
#endif

#if os(iOS)
    init(configurationManager: ConfigurationManager) {
        self.configurationManager = configurationManager
        setup()
    }
#elseif os(macOS)
    init(
        configurationManager: ConfigurationManager,
        grpcManager: GRPCManager
    ) {
        self.configurationManager = configurationManager
        self.grpcManager = grpcManager
        setupIsServingObserver()
        setupEnvironmentChangeObserver()
    }
#endif

    public func setup() {
        setupPeriodicRefresh()
        setupEnvironmentChangeObserver()
        updateFeatureFlags()
    }
}

#if os(iOS)
#elseif os(macOS)
private extension FeatureFlagsManager {
    func setupIsServingObserver() {
        grpcManager.$isServing
            .removeDuplicates()
            .sink { [weak self] isServing in
                guard let self, isServing else { return }
                Task { @MainActor in
                    self.updateFeatureFlags()
                }
            }
            .store(in: &cancellables)
    }
}
#endif

private extension FeatureFlagsManager {
    func setupEnvironmentChangeObserver() {
        configurationManager.environmentDidChange = { [weak self] in
            self?.updateFeatureFlags()
        }
    }

    func setupPeriodicRefresh() {
        Timer
            .publish(every: 600, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                Task { @MainActor in
                    self?.updateFeatureFlags()
                }
            }
            .store(in: &cancellables)
    }
}

private extension FeatureFlagsManager {
    func updateFeatureFlags() {
        Task {
#if os(iOS)
            guard let flags = try? currentEnvironment().featureFlags else { return }
            Task { @MainActor in
            }
#elseif os(macOS)
            guard let flags = try? await grpcManager.fetchFeatureFlags() else { return }
#endif
        }
    }
}
