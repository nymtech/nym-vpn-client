import Combine
import SwiftUI
import Logging
import AppSettings
#if os(iOS)
import MixnetLibrary
#elseif os(macOS)
import GRPCManager
#endif
import SystemMessageModels

public final class MessagesManager: ObservableObject {
    private let appSettings: AppSettings
#if os(macOS)
    private let grpcManager: GRPCManager
#endif
    private let logger = Logger(label: "SystemMessageManager")

    private var messages: [SnackBarMessage] = []
    private var timer: Timer?

    public static let shared = MessagesManager()

    @Published public var currentMessage: SnackBarMessage?

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

    nonisolated public func setup() {
        fetchSystemMessages()
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
        timer = Timer.scheduledTimer(withTimeInterval: 10, repeats: false) { [weak self] _ in
            self?.currentMessage = nil
        }
    }

    public func messageDidClose() {
        guard !messages.isEmpty else { return }
        messages.removeFirst()

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
                newMessages = try getSystemMessages().map {
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
