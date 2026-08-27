#if os(macOS)
import SwiftUI
import AppKit
import ConnectionManager
import ConnectionTypes
import GRPCManager
import ImpactGenerator
import Theme

@MainActor public final class GeoExclusionViewModel: ObservableObject {
    @ObservedObject var connectionManager: ConnectionManager
    @ObservedObject var grpcManager: GRPCManager

    @Binding private var path: NavigationPath
    private let impactGenerator: ImpactGenerator

    @Published var isEnabled = false
    @Published var isLoading = false
    @Published var failedToStart = false
    @Published var portText = "\(GeoExclusionConfig.defaultPort)"
    @Published var portError: String?
    @Published var portCopied = false
    @Published var serverCopied = false

    private(set) var listenPort = GeoExclusionConfig.defaultPort

    /// The loopback host shown in the Server row; the proxy always listens on 127.0.0.1.
    let serverAddress = "127.0.0.1"

    /// Beta: the excluded region list is hardcoded to China.
    let excludedCountryName = "geoExclusion.china".localizedString

    public init(
        path: Binding<NavigationPath>,
        connectionManager: ConnectionManager,
        grpcManager: GRPCManager,
        impactGenerator: ImpactGenerator
    ) {
        _path = path
        self.connectionManager = connectionManager
        self.grpcManager = grpcManager
        self.impactGenerator = impactGenerator
    }

    var proxyAddress: String {
        "127.0.0.1:\(listenPort)"
    }

    func navigateBack() {
        guard !path.isEmpty else { return }
        impactGenerator.softImpact()
        path.removeLast()
    }

    func navigateToSetup() {
        impactGenerator.softImpact()
        path.append(SettingLink.geoExclusionSetup(port: listenPort))
    }

    func loadState() async {
        isLoading = true
        let config = await grpcManager.config()?.geoExclusionConfig ?? GeoExclusionConfig()
        isEnabled = config.isEnabled
        listenPort = config.listenPort
        portText = "\(listenPort)"
        isLoading = false
    }

    func setEnabled(_ enabled: Bool) {
        isEnabled = enabled
        Task {
            do {
                try await grpcManager.setGeoExclusionEnabled(enabled)
                if enabled {
                    try await grpcManager.setGeoExclusionExcludedCountries(GeoExclusionConfig.defaultExcludedCountries)
                    try await grpcManager.setGeoExclusionListenPort(listenPort)
                }
                failedToStart = false
            } catch {
                failedToStart = true
                isEnabled = false
            }
        }
    }

    /// Called on every keystroke: strips non-digits (max 5 chars), then live-validates.
    func portTextChanged() {
        let filtered = GeoExclusionConfig.sanitizedPortText(portText)
        if filtered != portText {
            portText = filtered
        }
        portError = portValidationMessage(for: portText)
    }

    /// Commit on submit / focus loss. Pushes a valid changed port; reverts to the last
    /// valid value when the field is left invalid (the live error already explained why).
    func commitPort() {
        let trimmed = portText.trimmingCharacters(in: .whitespaces)
        guard let port = UInt16(trimmed), GeoExclusionConfig.isValidPort(port) else {
            portText = "\(listenPort)"
            portError = nil
            return
        }
        portError = nil
        guard port != listenPort else { return }
        Task {
            do {
                try await grpcManager.setGeoExclusionListenPort(port)
                listenPort = port
                portText = "\(port)"
            } catch {
                portText = "\(listenPort)"
                portError = nil
            }
        }
    }

    func copyPort() {
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString("\(listenPort)", forType: .string)
        withAnimation {
            guard !portCopied else { return }
            portCopied = true
            Task { @MainActor in
                try? await Task.sleep(for: .seconds(3))
                portCopied = false
            }
        }
    }

    func copyServer() {
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(serverAddress, forType: .string)
        withAnimation {
            guard !serverCopied else { return }
            serverCopied = true
            Task { @MainActor in
                try? await Task.sleep(for: .seconds(3))
                serverCopied = false
            }
        }
    }
}

private extension GeoExclusionViewModel {
    func portValidationMessage(for text: String) -> String? {
        switch GeoExclusionConfig.validate(portText: text) {
        case .valid, .empty:
            return nil
        case .outOfRange:
            return "geoExclusion.port.outOfRange".localizedString
        case .forbidden:
            return String(format: "geoExclusion.port.forbidden".localizedString, "\(GeoExclusionConfig.forbiddenPort)")
        }
    }
}
#endif
