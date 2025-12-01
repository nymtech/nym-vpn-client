import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManagerTypes
import FeatureFlagsManager
import GatewayManager
import ImpactGenerator
import Theme
import UIComponents

public struct GatewayCell: View {
    private let server: GatewayNode
    private let hopType: HopType
    private let isSearching: Bool

    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var gatewayManager: GatewayManager
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var featureFlagsManager: FeatureFlagsManager
    @Binding private var path: NavigationPath
    @Binding private var scrollToModel: GatewayScrollToModel
    @State private var isHovered = false
    @State private var isSelected: Bool
    private var infoButtonTapCompletion: (@Sendable @MainActor (GatewayNode) -> Void)?

    private var shouldShowQuic: Bool {
        hopType == .entry
        && connectionManager.connectionType == .wireguard
        && appSettings.isQuicEnabled
    }

    private var shouldShowStreaming: Bool {
        hopType == .exit
        && server.isResidentialAvailable
    }

    public init(
        server: GatewayNode,
        type: HopType,
        path: Binding<NavigationPath>,
        scrollToModel: Binding<GatewayScrollToModel>,
        isSearching: Bool = false,
        infoButtonTapCompletion: (@Sendable @MainActor (GatewayNode) -> Void)?
    ) {
        self.server = server
        self.hopType = type
        self.isSearching = isSearching
        _path = path
        _scrollToModel = scrollToModel
        self.infoButtonTapCompletion = infoButtonTapCompletion

        let unwrappedScrollToModel = scrollToModel.wrappedValue
        let shouldSelect = unwrappedScrollToModel.serverId == server.id && unwrappedScrollToModel.isServer
        _isSelected = State(initialValue: shouldSelect)
    }

    public var body: some View {
        HStack(spacing: 0) {
            serverInfo()
            Spacer()
                .frame(width: 16)
            if shouldShowQuic {
                QuicLabel()
            } else if shouldShowStreaming {
                StreamingIcon()
            }

            infoButton()
                .contentShape(Rectangle())
                .onTapGesture {
                    infoButtonTapAction()
                }
                .accessibilityAction {
                    infoButtonTapAction()
                }
        }
        .background(isHovered ? NymColor.backgroundHover : NymColor.background)
        .onHover { newValue in
            isHovered = newValue
        }
    }
}

private extension GatewayCell {
    func serverInfo() -> some View {
        HStack(spacing: 0) {
            selectionMarkerView()
            scoreImage()
            serverDetails()
            Spacer()
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(server.name ?? server.id)")
        .accessibilityValue(isSelected ? "selected".localizedString : "")
        .accessibilityAddTraits([.isButton])
        .contentShape(Rectangle())
        .onTapGesture {
            tapAction()
        }
        .accessibilityAction {
            tapAction()
        }
    }
}

private extension GatewayCell {
    func tapAction() {
        ImpactGenerator.shared.softImpact()
        switch hopType {
        case .entry:
            connectionManager.entryGateway = .gateway(server.id)
        case .exit:
            connectionManager.exitRouter = .gateway(server.id)
        }
        path = .init()
    }

    func infoButtonTapAction() {
        ImpactGenerator.shared.softImpact()
        infoButtonTapCompletion?(server)
    }
}

private extension GatewayCell {
    @ViewBuilder
    func selectionMarkerView() -> some View {
        if isSelected {
            SelectionMarker()
        }
    }

    func scoreImage() -> some View {
        GenericImage(imageName: scoreImageName())
            .frame(width: 16, height: 16)
            .padding(20)
    }

    func serverDetails() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            serverTitle()
            Spacer()
                .frame(height: 4)
            serverSubtitle()
        }
    }

    func serverTitle() -> some View {
        Text(server.name ?? server.id)
            .lineLimit(1)
            .foregroundStyle(NymColor.primary)
            .textStyle(.Body.Large.regular)
    }

    func serverSubtitleString() -> String {
        if isSearching,
           let countryCode = server.location?.twoLetterIsoCountryCode,
           let country = gatewayManager.localizedCountry(with: countryCode),
           let city = server.location?.city {
            "\(city), \(country.name), \(server.id)"
        } else {
            server.location?.city ?? server.id
        }
    }

    func serverSubtitle() -> some View {
        Text(serverSubtitleString())
            .lineLimit(1)
            .truncationMode(.middle)
            .foregroundStyle(NymColor.gray1)
            .textStyle(.Body.Small.regular)
    }

    func infoButton() -> some View {
        GenericImage(systemImageName: "info.circle", allowsHover: true)
            .frame(width: 18, height: 18)
            .padding(19)
    }
}

extension GatewayCell {
    func scoreImageName() -> String {
        let score: GatewayNodeScore?
        switch connectionManager.connectionType {
        case .mixnet5hop:
            score = server.performance?.mixnetScore
        case .wireguard:
            score = server.performance?.score
        }
        guard let score else { return "scoreLow"}
        switch score {
        case .low:
            return "scoreLow"
        case .medium:
            return "scoreMedium"
        case .high:
            return "scoreHigh"
        case .offline, .noScore:
            return "scoreOffline"
        }
    }
}
