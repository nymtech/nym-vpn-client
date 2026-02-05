import Combine
import SwiftUI
import Logging
import AppSettings
import ConfigurationManager
#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif
import MessageModels

@MainActor public final class MessagesManager: ObservableObject {
    private let appSettings: AppSettings
#if os(iOS)
    private let configurationManager: ConfigurationManager
#endif
#if os(macOS)
    private let grpcManager: GRPCManager
#endif
    private let logger = Logger(label: "MessagesManager")

    private var messages: [SnackBarMessage] = []
    private var timer: Timer?
#if os(iOS)
    public static let shared = MessagesManager(appSettings: .shared, configurationManager: .shared)
#elseif os(macOS)
    public static let shared = MessagesManager(appSettings: .shared, grpcManager: .shared)
#endif
    @Published public var currentMessage: SnackBarMessage?

#if os(iOS)
    init(appSettings: AppSettings, configurationManager: ConfigurationManager) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
    }
#elseif os(macOS)
    init(
        appSettings: AppSettings,
        grpcManager: GRPCManager
    ) {
        self.appSettings = appSettings
        self.grpcManager = grpcManager
    }
#endif

    nonisolated public func setup() {
        Task { @MainActor [weak self] in
            self?.fetchSystemMessages()
        }
    }

    public func processMessages() {
        timer?.invalidate()

        guard !messages.isEmpty,
              let message = messages.first,
              !message.text.isEmpty
        else {
            return
        }

        currentMessage = message
        timer = Timer.scheduledTimer(withTimeInterval: 20, repeats: false) { [weak self] _ in
            Task { @MainActor in
                self?.currentMessage = nil
            }
        }
    }

    public func messageDidClose() {
        guard !messages.isEmpty else { return }
        messages.removeFirst()

        processMessages()
    }

    public func addAndProcess(_ message: SnackBarMessage) {
        messages.append(message)
        processMessages()
    }
}

// MARK: - System messages -
private extension MessagesManager {
    func fetchSystemMessages() {
        Task {
            do {
                let newMessages: [NymNetworkMessage]
#if os(iOS)
                newMessages = configurationManager.networkEnv.systemMessages().map {
                    NymNetworkMessage(name: $0.name, message: $0.message, properties: $0.properties)
                }
#elseif os(macOS)
                newMessages = try await grpcManager.fetchSystemMessages()
#endif
                await updateMessages(with: newMessages)
            } catch {
                logger.error("Failed to fetch system messages: \(error)")
            }
        }
    }

    func updateMessages(with newMessages: [NymNetworkMessage]) async {
        await MainActor.run {
            let messages = newMessages.map {
                SnackBarMessage(text: $0.message, style: .info)
            }
            self.messages.append(contentsOf: messages)
        }
    }
}
