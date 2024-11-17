import SwiftUI
import Logging
import AppSettings
#if os(iOS)
import MixnetLibrary
#elseif os(macOS)
import GRPCManager
#endif
import SystemMessageModels

public final class SystemMessageManager: ObservableObject {
    private let appSettings: AppSettings
#if os(macOS)
    private let grpcManager: GRPCManager
#endif
    private let logger = Logger(label: "SystemMessageManager")

    public static let shared = SystemMessageManager()

    @Published public var messages: [NymNetworkMessage] = []

#if os(iOS)
    init(appSettings: AppSettings = .shared) {
        self.appSettings = appSettings
    }
#elseif os(macOS)
    init(
        appSettings: AppSettings = .shared,
        grpcManager: GRPCManager = .shared
    ) {
        self.appSettings = appSettings
        self.grpcManager = grpcManager
    }
#endif

    public func fetchMessages() {
        Task(priority: .background) {
            do {
#if os(iOS)
                messages = try fetchSystemMessages(networkName: appSettings.currentEnv).map {
                    NymNetworkMessage(name: $0.name, message: $0.message, properties: $0.properties)
                }
#elseif os(macOS)
                messages = try await grpcManager.fetchSystemMessages()
#endif
            } catch {
                logger.error("Failed to fetch system messages: \(error)")
            }
        }
    }
}
