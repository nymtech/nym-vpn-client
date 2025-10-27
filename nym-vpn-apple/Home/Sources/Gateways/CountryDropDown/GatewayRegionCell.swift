import SwiftUI
import ConnectionManager
import ConnectionTypes
import CountriesManagerTypes
import GatewayManager
import ImpactGenerator
import Theme
import UIComponents

public struct GatewayRegionCell: View {
    private let hopType: HopType
    private let country: NymCountry
    private let region: String
    private let servers: [GatewayNode]
    @EnvironmentObject private var gatewayManager: GatewayManager
    @Binding private var entryGateway: EntryGateway
    @Binding private var exitRouter: ExitRouter
    @Binding private var path: NavigationPath
    @Binding private var scrollToModel: GatewayScrollToModel
    @State private var isExpanded: Bool
    @State private var isHovered = false
    @State private var isRegionSelected = false
    private var infoButtonTapCompletion: (@Sendable @MainActor (GatewayNode) -> Void)?

    public var body: some View {
        VStack(spacing: 0) {
            regionCell()
            if isExpanded {
                ForEach(servers, id: \.id) { server in
                    GatewayCell(
                        server: server,
                        type: hopType,
                        path: $path,
                        scrollToModel: $scrollToModel,
                        infoButtonTapCompletion: { server in
                            infoButtonTapCompletion?(server)
                        }
                    )
                    .id(GatewayScrollToModel.server(id: server.id).scrollToIdentifier)
                }
            }
        }
    }

    public init(
        hopType: HopType,
        country: NymCountry,
        region: String,
        servers: [GatewayNode],
        infoButtonTapCompletion: (@Sendable @MainActor (GatewayNode) -> Void)?,
        path: Binding<NavigationPath>,
        entryGateway: Binding<EntryGateway>,
        exitRouter: Binding<ExitRouter>,
        scrollToModel: Binding<GatewayScrollToModel>
    ) {
        self.hopType = hopType
        self.country = country
        self.region = region
        self.servers = servers
        self.infoButtonTapCompletion = infoButtonTapCompletion
        _path = path
        _entryGateway = entryGateway
        _exitRouter = exitRouter
        _scrollToModel = scrollToModel

        let unwrappedScrollToModel = scrollToModel.wrappedValue
        let selectedServer = servers.first { $0.id == unwrappedScrollToModel.serverId }
        let shouldExpand = unwrappedScrollToModel.shouldExpand(
            countryCode: country.code,
            region: region,
            server: selectedServer
        )
        _isExpanded = State(initialValue: shouldExpand)
        let shouldSelect = unwrappedScrollToModel.region == region && unwrappedScrollToModel.isRegion
        _isRegionSelected = State(initialValue: shouldSelect)
    }
}

public extension GatewayRegionCell {
    @ViewBuilder
    func regionCell() -> some View {
        HStack(spacing: 0) {
            HStack(spacing: 0) {
                isSelectedMarker()
                FlagImage(countryCode: country.code)
                    .padding(EdgeInsets(top: 0, leading: isRegionSelected ? 12 : 16, bottom: 0, trailing: 16))
                VStack(alignment: .leading, spacing: 0) {
                    regionNameTitle()
                    serverCountNumberSubtitle()
                }
                Spacer()
            }
            .accessibilityElement(children: .combine)
            .accessibilityLabel("\(region) \(servers.count) \("servers".localizedString)")
            .accessibilityValue(isRegionSelected ? "selected".localizedString : "")
            .accessibilityAddTraits([.isButton])
            .contentShape(Rectangle())
            .onTapGesture {
                stateTapAction()
            }
            .accessibilityAction {
                stateTapAction()
            }
            HStack(spacing: 0) {
                lineSeparator()
                arrowDropDown()
            }
            .accessibilityElement(children: .combine)
            .accessibilityLabel("gatewaySelector.expandServers".localizedString)
            .accessibilityAddTraits([.isButton])
            .contentShape(Rectangle())
            .onTapGesture {
                expandTapAction()
            }
            .accessibilityAction {
                expandTapAction()
            }
        }
        .onHover { newValue in
            isHovered = newValue
        }
        .background {
            isHovered ? NymColor.elevationHover : NymColor.elevation.opacity(0.6)
        }
    }

    @ViewBuilder
    func isSelectedMarker() -> some View {
        if isRegionSelected {
            SelectionMarker()
        }
    }

    func regionNameTitle() -> some View {
        Text(region)
            .foregroundStyle(NymColor.primary)
            .textStyle(.Body.Large.regular)
    }

    func serverCountNumberSubtitle() -> some View {
        Text("\(servers.count) \("servers".localizedString)")
            .foregroundStyle(NymColor.gray1)
            .textStyle(.Body.Small.regular)
    }

    func lineSeparator() -> some View {
        Rectangle()
            .foregroundColor(NymColor.gray2)
            .frame(width: 1, height: 42)
            .padding(0)
    }

    func arrowDropDown() -> some View {
        GenericImage(imageName: "arrowDropDown")
            .frame(width: 24, height: 24)
            .padding(16)
            .rotationEffect(.degrees(isExpanded ? 180 : 0))
            .animation(.easeInOut, value: isExpanded)
    }
}

public extension GatewayRegionCell {
    func updateIsRegionSelected() {
        switch scrollToModel {
        case let .region(countryCode: _, region: countryRegion):
            isRegionSelected = countryRegion == region
        default:
            isRegionSelected = false
        }
    }

    func expandTapAction() {
        ImpactGenerator.shared.softImpact()
        isExpanded.toggle()
    }

    func stateTapAction() {
        ImpactGenerator.shared.softImpact()
        switch hopType {
        case .entry:
            entryGateway = .region(countryCode: country.code, region: region)
        case .exit:
            exitRouter = .region(countryCode: country.code, region: region)
        }
        path = .init()
    }
}
