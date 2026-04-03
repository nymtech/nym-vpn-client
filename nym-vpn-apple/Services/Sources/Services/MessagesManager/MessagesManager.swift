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
import Theme

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
    private var cancellables = Set<AnyCancellable>()
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
            currentMessage = nil
            return
        }

        guard currentMessage?.text != message.text
                || currentMessage?.priority != message.priority
        else {
            return
        }

        currentMessage = message

        guard message.priority != .low else { return }
        timer = Timer.scheduledTimer(withTimeInterval: 20, repeats: false) { [weak self] _ in
            Task { @MainActor in
                self?.currentMessage = nil
            }
        }
    }

    public func messageDidClose() {
        timer?.invalidate()
        guard !messages.isEmpty else { return }
        let removed = messages.removeFirst()
        removed.closeAction?()
        currentMessage = nil

        Task { @MainActor [weak self] in
            try? await Task.sleep(for: .seconds(0.5))
            self?.processMessages()
        }
    }

    public func addAndProcess(_ message: SnackBarMessage) {
        let insertIndex = messages.firstIndex { $0.priority < message.priority } ?? messages.endIndex
        messages.insert(message, at: insertIndex)
        processMessages()
    }

    public func hasMessages(withPriority priority: BannerPriority) -> Bool {
        messages.contains { $0.priority == priority } || currentMessage?.priority == priority
    }

    public func removeMessages(withPriority priority: BannerPriority) {
        messages.removeAll { $0.priority == priority }
        if currentMessage?.priority == priority {
            currentMessage = nil
            processMessages()
        }
    }
}

// MARK: - Passphrase banner -
extension MessagesManager {
    public func configurePassphraseBanner(ctaAction: @escaping () -> Void) {
        appSettings.$isPassphraseStoredPublisher
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] isStored in
                MainActor.assumeIsolated {
                    if isStored {
                        self?.removeMessages(withPriority: .low)
                    } else {
                        self?.enqueuePassphraseBannerIfNeeded(ctaAction: ctaAction)
                    }
                }
            }
            .store(in: &cancellables)
    }

    private func enqueuePassphraseBannerIfNeeded(ctaAction: @escaping () -> Void) {
        guard !hasMessages(withPriority: .low) else { return }

        let titleText = "passphraseOverlay.connected".localizedString
            + "\n"
            + "passphraseOverlay.secureAccess".localizedString
            + " 🔒"

        addAndProcess(
            SnackBarMessage(
                text: titleText,
                style: .passphrase,
                ctaText: "passphraseOverlay.backup.line1".localizedString
                    + "\n"
                    + "passphraseOverlay.backup.line2".localizedString,
                ctaAction: ctaAction,
                priority: .low
            )
        )
    }
}

// MARK: - Expiry banner -
extension MessagesManager {
    public func enqueueExpiryBanner(
        subtitle: String,
        ctaAction: @escaping () -> Void,
        closeAction: @escaping () -> Void
    ) {
        guard !hasMessages(withPriority: .high) else { return }

        addAndProcess(
            SnackBarMessage(
                text: "\("planExpiresOn".localizedString):",
                style: .expiry,
                subtitle: subtitle,
                ctaText: "settings.account.renewNow.line1".localizedString
                    + "\n"
                    + "settings.account.renewNow.line2".localizedString,
                ctaAction: ctaAction,
                closeAction: closeAction,
                priority: .high
            )
        )
    }
}

// MARK: - System messages -
private extension MessagesManager {
    func fetchSystemMessages() {
        Task {
            do {
                let newMessages: [NymNetworkMessage]
#if os(iOS)
                newMessages = configurationManager.networkEnv?
                    .systemMessages()
                    .compactMap { NymNetworkMessage(name: $0.name, message: $0.message, properties: $0.properties) }
                ?? []
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
