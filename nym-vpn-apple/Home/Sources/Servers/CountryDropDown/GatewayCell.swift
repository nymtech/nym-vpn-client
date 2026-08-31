import SwiftUI
import AppSettings
import ConnectionManager
import ConnectionTypes
import FeatureFlagsManager
import GatewayManager
import ImpactGenerator
import Theme
import UIComponents

public struct GatewayCell: View {
    private let server: GatewayNode
    private let hopType: HopType
    private let isSearching: Bool
    private let bottomCornerRadius: CGFloat

    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var gatewayManager: GatewayManager
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var featureFlagsManager: FeatureFlagsManager
    @EnvironmentObject private var favoritesState: ServersFavoritesState
    @Binding private var path: NavigationPath
    @Binding private var scrollToModel: GatewayScrollToModel
    @State private var isButtonHovered = false
    @State private var isAccessoryHovered = false
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
        bottomCornerRadius: CGFloat = 0,
        infoButtonTapCompletion: (@Sendable @MainActor (GatewayNode) -> Void)?
    ) {
        self.server = server
        self.hopType = type
        self.isSearching = isSearching
        self.bottomCornerRadius = bottomCornerRadius
        _path = path
        _scrollToModel = scrollToModel
        self.infoButtonTapCompletion = infoButtonTapCompletion

        let unwrappedScrollToModel = scrollToModel.wrappedValue
        let shouldSelect = unwrappedScrollToModel.serverId == server.id && unwrappedScrollToModel.isServer
        _isSelected = State(initialValue: shouldSelect)
    }

    public var body: some View {
        HStack(spacing: 0) {
            HStack(spacing: 0) {
                serverInfo()
                if shouldShowQuic {
                    QuicLabel()
                        .padding(.trailing, NymSpacing.small)
                } else if shouldShowStreaming {
                    StreamingIcon()
                        .padding(.trailing, NymSpacing.small)
                }
            }

            FavoriteStarButton(
                isFavorite: favoritesState.isFavorite(.gateway(server.id)),
                action: { favoritesState.toggleFavorite(.gateway(server.id)) }
            )
            .padding(.trailing, NymSpacing.small)

            infoButton()
                .onHover { newValue in
                    isAccessoryHovered = newValue
                }
                .contentShape(Rectangle())
                .onTapGesture {
                    infoButtonTapAction()
                }
                .accessibilityAction {
                    infoButtonTapAction()
                }
        }
        .frame(minHeight: 64)
        .background(isButtonHovered ? Color.Nym.background.opacity(0.3) : Color.clear)
        .overlay {
            UnevenRoundedRectangle(
                topLeadingRadius: 0,
                bottomLeadingRadius: bottomCornerRadius,
                bottomTrailingRadius: bottomCornerRadius,
                topTrailingRadius: 0
            )
            .inset(by: 0.5)
            .stroke(isSelected ? Color.Nym.primary : .clear, lineWidth: 1)
            .allowsHitTesting(false)
        }
        .animation(.default, value: isSelected)
        .onHover { newValue in
            isButtonHovered = newValue
        }
    }
}

private extension GatewayCell {
    func serverInfo() -> some View {
        HStack(spacing: 0) {
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
            connectionManager.setEntryGateway(.gateway(server.id))
        case .exit:
            connectionManager.applyExplicitExit(.gateway(server.id))
        }
        path = .init()
    }

    func infoButtonTapAction() {
        ImpactGenerator.shared.softImpact()
        infoButtonTapCompletion?(server)
    }
}

private extension GatewayCell {
    func scoreImage() -> some View {
        GenericImage(imageName: scoreImageName())
            .frame(width: 16, height: 16)
            .padding(.leading, NymSpacing.large)
            .padding(.trailing, NymSpacing.medium)
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
            .foregroundStyle(Color.Nym.textPrimary)
            .nymTextStyle(.bodyLarge)
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
            .foregroundStyle(Color.Nym.textSecondary)
            .nymTextStyle(.bodySmall)
    }

    func infoButton() -> some View {
        Image(systemName: "chevron.right")
            .font(.system(size: 14, weight: .semibold))
            .foregroundStyle(isAccessoryHovered ? Color.Nym.textPrimary : Color.Nym.textSecondary)
            .frame(width: 24, height: 24)
            .padding(.trailing, NymSpacing.large)
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
