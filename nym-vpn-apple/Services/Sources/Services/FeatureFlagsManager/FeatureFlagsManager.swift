import Foundation
import Combine
import AppSettings
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
    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager

    private var cancellables = Set<AnyCancellable>()

#if os(iOS)
    public static let shared = FeatureFlagsManager(configurationManager: .shared, appSettings: .shared)
#elseif os(macOS)
    public static let shared = FeatureFlagsManager(
        configurationManager: .shared,
        grpcManager: .shared,
        appSettings: .shared
    )
#endif

#if os(iOS)
    init(configurationManager: ConfigurationManager, appSettings: AppSettings) {
        self.configurationManager = configurationManager
        self.appSettings = appSettings
        setup()
    }
#elseif os(macOS)
    init(
        configurationManager: ConfigurationManager,
        grpcManager: GRPCManager,
        appSettings: AppSettings
    ) {
        self.configurationManager = configurationManager
        self.grpcManager = grpcManager
        self.appSettings = appSettings
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
        Task { [weak self] in
#if os(iOS)
            guard let flags = self?.configurationManager.networkEnv?.current().featureFlags else { return }
#elseif os(macOS)
            guard let flags = try? await self?.grpcManager.fetchFeatureFlags() else { return }
#endif
        }
    }
}
