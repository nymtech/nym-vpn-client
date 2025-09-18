import Foundation
import Combine
import FeatureFlagModels
#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif

public final class FeatureFlagsManager: ObservableObject {
#if os(macOS)
    private let grpcManager: GRPCManager
#endif
    private var featureFlags: [FeatureFlag]
    private var cancellables = Set<AnyCancellable>()

    public static let shared = FeatureFlagsManager()

    public var isDomainFrontingEnabled: Bool {
        featureFlags.contains(where: { $0.name == "domain_fronting.enabled" && $0.isEnabled })
    }

    public var isQuicEnabled: Bool {
        featureFlags.contains(where: { $0.name == "quic.enabled" && $0.isEnabled })
    }

#if os(iOS)
    init(featureFlags: [FeatureFlag] = [FeatureFlag]()) {
        self.featureFlags = featureFlags
        setup()
    }
#elseif os(macOS)
    init(
        grpcManager: GRPCManager = GRPCManager.shared,
        featureFlags: [FeatureFlag] = [FeatureFlag]()
    ) {
        self.grpcManager = grpcManager
        self.featureFlags = featureFlags
//        setupIsServingObserver()
    }
#endif

    public func setup() {
        Task {
            await updateFeatureFlags()
        }
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

