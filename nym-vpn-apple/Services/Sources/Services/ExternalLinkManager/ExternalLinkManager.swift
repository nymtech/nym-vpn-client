import AuthenticationServices
import Foundation

#if os(iOS)
import UIKit
#endif

#if os(macOS)
import AppKit
#endif

import Constants

@MainActor public final class ExternalLinkManager: NSObject, ObservableObject {
    public static let shared = ExternalLinkManager()

    public var deeplinkHandler: ((URL) async -> Void)?

    private var currentAuthSession: ASWebAuthenticationSession?

#if os(iOS)
    @Published public var inAppSafariURL: InAppSafariURL?

    public func openExternalURL(urlString: String?) throws {
        guard let urlString, let url = URL(string: urlString)
        else {
            throw GeneralNymError.invalidUrl
        }
        openExternalURL(url)
    }

    public func openExternalURL(_ url: URL) {
        UIApplication.shared.open(url)
    }

    public func openInAppBrowser(urlString: String?) throws {
        guard let urlString, let url = URL(string: urlString)
        else {
            throw GeneralNymError.invalidUrl
        }
        openInAppBrowser(url)
    }

    public func openInAppBrowser(_ url: URL) {
        guard let scheme = url.scheme?.lowercased(),
              scheme == "https"
        else {
            openExternalURL(url)
            return
        }
        inAppSafariURL = InAppSafariURL(url: url)
    }

    public func dismissActiveWebCheckoutSessions() {
        currentAuthSession?.cancel()
        currentAuthSession = nil
        inAppSafariURL = nil
    }
#endif

#if os(macOS)
    public func openExternalURL(urlString: String?) throws {
        guard let urlString, let url = URL(string: urlString)
        else {
            throw GeneralNymError.invalidUrl
        }
        openExternalURL(url)
    }

    public func openExternalURL(_ url: URL) {
        NSWorkspace.shared.open(url)
    }
#endif

    public func presentPrivyAuthSession(urlString: String?) async throws {
        guard let urlString, let url = URL(string: urlString)
        else {
            throw GeneralNymError.invalidUrl
        }
        try await presentPrivyAuthSession(url)
    }

    public func presentPrivyAuthSession(_ url: URL) async throws {
#if os(macOS)
        openExternalURL(url)
#else
        let callbackURL: URL = try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                let session = ASWebAuthenticationSession(
                    url: url,
                    callbackURLScheme: Constants.appUrlScheme.rawValue
                ) { callbackURL, error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else if let callbackURL {
                        continuation.resume(returning: callbackURL)
                    } else {
                        continuation.resume(throwing: GeneralNymError.invalidUrl)
                    }
                }
                session.prefersEphemeralWebBrowserSession = true
                session.presentationContextProvider = self
                currentAuthSession = session
                if !session.start() {
                    currentAuthSession = nil
                    continuation.resume(throwing: GeneralNymError.invalidUrl)
                }
            }
        } onCancel: { [weak self] in
            Task { @MainActor in
                self?.currentAuthSession?.cancel()
                self?.currentAuthSession = nil
            }
        }
        currentAuthSession = nil
        if let deeplinkHandler {
            await deeplinkHandler(callbackURL)
        }
        dismissActiveWebCheckoutSessions()
#endif
    }
}

extension ExternalLinkManager: ASWebAuthenticationPresentationContextProviding {
    nonisolated public func presentationAnchor(
        for session: ASWebAuthenticationSession
    ) -> ASPresentationAnchor {
        MainActor.assumeIsolated {
#if os(iOS)
            let scene = UIApplication.shared.connectedScenes
                .compactMap { $0 as? UIWindowScene }
                .first { $0.activationState == .foregroundActive }
                ?? UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first
            return scene?.keyWindow ?? scene?.windows.first ?? UIWindow()
#elseif os(macOS)
            return NSApplication.shared.keyWindow
                ?? NSApplication.shared.windows.first
                ?? ASPresentationAnchor()
#endif
        }
    }
}
