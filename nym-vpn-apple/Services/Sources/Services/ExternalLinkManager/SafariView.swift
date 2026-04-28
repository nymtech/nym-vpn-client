#if os(iOS)
import SafariServices
import SwiftUI

public struct InAppSafariURL: Identifiable, Equatable {
    public let id = UUID()
    public let url: URL

    public init(url: URL) {
        self.url = url
    }
}

public struct SafariView: UIViewControllerRepresentable {
    private let url: URL

    public init(url: URL) {
        self.url = url
    }

    public func makeUIViewController(context: Context) -> SFSafariViewController {
        let configuration = SFSafariViewController.Configuration()
        configuration.entersReaderIfAvailable = false
        configuration.barCollapsingEnabled = true

        let viewController = SFSafariViewController(url: url, configuration: configuration)
        viewController.dismissButtonStyle = .done
        viewController.modalPresentationStyle = .pageSheet
        return viewController
    }

    public func updateUIViewController(_ uiViewController: SFSafariViewController, context: Context) {}
}
#endif
