import SwiftUI
import ConnectionManager
import CountriesManager
import CountriesManagerTypes
import Theme

public struct GatewayCell: View {
    private let server: GatewayNode
    private let hopType: HopType
    private let isSearching: Bool

    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var countriesManager: CountriesManager
    @Binding private var path: NavigationPath
    @Binding private var isServerModalDisplayed: Bool
    @Binding private var serverInfoModalServer: GatewayNode?
    @State private var isHovered = false

    public init(
        server: GatewayNode,
        type: HopType,
        path: Binding<NavigationPath>,
        isServerModalDisplayed: Binding<Bool>,
        serverInfoModalServer: Binding<GatewayNode?>,
        isSearching: Bool = false
    ) {
        self.server = server
        self.hopType = type
        self.isSearching = isSearching
        _path = path
        _isServerModalDisplayed = isServerModalDisplayed
        _serverInfoModalServer = serverInfoModalServer
    }

    public var body: some View {
        HStack(spacing: 0) {
            HStack(spacing: 0) {
                selectionMarkerView()
                scoreImage()
                serverDetails()
            }
            .contentShape(Rectangle())
            .onTapGesture {
                switch hopType {
                case .entry:
                    connectionManager.entryGateway = .gateway(server)
                case .exit:
                    connectionManager.exitRouter = .gateway(server)
                }
                path = .init()
            }

            Spacer()
                .frame(width: 16)
            lineSeparator()
            infoButton()
                .contentShape(Rectangle())
                .onTapGesture {
                    serverInfoModalServer = server
                    withAnimation {
                        isServerModalDisplayed.toggle()
                    }
                }
        }
        .background(isHovered ? NymColor.backgroundHover : NymColor.background)
        .onHover { newValue in
            isHovered = newValue
        }
    }
}

extension GatewayCell {
    func isSelected() -> Bool {
        switch hopType {
        case .entry:
            connectionManager.entryGateway.gatewayId == server.id && connectionManager.entryGateway.isGateway
        case .exit:
            connectionManager.exitRouter.gatewayId == server.id && connectionManager.exitRouter.isGateway
        }
    }

    @ViewBuilder
    func selectionMarkerView() -> some View {
        if isSelected() {
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
        Text(server.moniker ?? server.id)
            .lineLimit(1)
            .foregroundStyle(NymColor.primary)
            .textStyle(.BodyLegacy.Large.regular)
    }

    func serverSubtitleString() -> String {
        if isSearching, let country = countriesManager.country(with: server.countryCode) {
            "\(country.name), \(server.id)"
        } else {
            server.id
        }
    }

    func serverSubtitle() -> some View {
        Text(serverSubtitleString())
            .lineLimit(1)
            .truncationMode(.middle)
            .foregroundStyle(NymColor.gray1)
            .textStyle(.BodyLegacy.Small.primary)
    }

    func lineSeparator() -> some View {
        Rectangle()
            .foregroundColor(NymColor.gray2)
            .frame(width: 1, height: 42)
            .padding(0)
    }

    func infoButton() -> some View {
        GenericImage(systemImageName: "info.circle", allowsHover: true)
            .frame(width: 18, height: 18)
            .padding(22)
    }
}

extension GatewayCell {
    func scoreImageName() -> String {
        let score: GatewayNodeScore
        switch connectionManager.connectionType {
        case .mixnet5hop:
            score = server.mixnetScore
        case .wireguard:
            score = server.wgScore
        }
        switch score {
        case .low, .noScore, .unrecognized:
            return "scoreLow"
        case .medium:
            return "scoreMedium"
        case .high:
            return "scoreHigh"
        }
    }
}
