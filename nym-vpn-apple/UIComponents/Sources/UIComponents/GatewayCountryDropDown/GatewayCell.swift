import SwiftUI
import ConnectionManager
import CountriesManagerTypes
import Theme

public struct GatewayCell: View {
    private let server: GatewayNode
    private let hopType: HopType

    @EnvironmentObject private var connectionManager: ConnectionManager
    @Binding private var path: NavigationPath
    @Binding private var isServerModalDisplayed: Bool
    @Binding private var serverInfoModalServer: GatewayNode?

    public init(
        server: GatewayNode,
        type: HopType,
        path: Binding<NavigationPath>,
        isServerModalDisplayed: Binding<Bool>,
        serverInfoModalServer: Binding<GatewayNode?>
    ) {
        self.server = server
        self.hopType = type
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
                    isServerModalDisplayed.toggle()
                }
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
            serverSubtitle()
        }
    }

    func serverTitle() -> some View {
        Text(server.moniker ?? server.id)
            .lineLimit(1)
            .foregroundStyle(NymColor.primary)
            .textStyle(.BodyLegacy.Large.regular)
    }

    func serverSubtitle() -> some View {
        Text(server.id)
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
        GenericImage(systemImageName: "questionmark.circle")
            .frame(width: 24, height: 24)
            .padding(16)
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
