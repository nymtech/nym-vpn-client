import Foundation
import Combine
import ConfigurationManager
import FeatureFlagModels
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
    private var featureFlags: [FeatureFlag]
    private var cancellables = Set<AnyCancellable>()

#if os(iOS)
    public static let shared = FeatureFlagsManager(featureFlags: [], configurationManager: .shared)
#elseif os(macOS)
    public static let shared = FeatureFlagsManager(
        featureFlags: [],
        configurationManager: .shared,
        grpcManager: .shared
    )
#endif

    public var isStealthAPIEnabled: Bool {
        featureFlags.contains(where: { $0.name == "domain_fronting.enabled" && $0.isEnabled })
    }

    public var isQuicEnabled: Bool {
        featureFlags.contains(where: { $0.name == "quic.enabled" && $0.isEnabled })
    }

#if os(iOS)
    init(featureFlags: [FeatureFlag], configurationManager: ConfigurationManager) {
        self.featureFlags = featureFlags
        self.configurationManager = configurationManager
        setup()
    }
#elseif os(macOS)
    init(
        featureFlags: [FeatureFlag],
        configurationManager: ConfigurationManager,
        grpcManager: GRPCManager
    ) {
        self.featureFlags = featureFlags
        self.configurationManager = configurationManager
        self.grpcManager = grpcManager
        setupIsServingObserver()
        setupEnvironmentChangeObserver()
    }
#endif

    public func setup() {
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
}

private extension FeatureFlagsManager {
    @MainActor func updateFeatureFlags() {
        Task {
#if os(iOS)
            guard let flags = try? currentEnvironment().featureFlags else { return }
            featureFlags = flags.toFeatureFlagList()
#elseif os(macOS)
            guard let flags = try? await grpcManager.fetchFeatureFlags() else { return }
            featureFlags = flags
#endif
        }
    }
}
