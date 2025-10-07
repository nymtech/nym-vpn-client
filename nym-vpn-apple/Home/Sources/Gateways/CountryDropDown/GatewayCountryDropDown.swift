import SwiftUI
import ConnectionManager
import ConnectionTypes
import CountriesManagerTypes
import GatewayManager
import ImpactGenerator
import Theme
import UIComponents

public struct GatewayCountryDropDown: View {
    private let country: Country
    private let regions: [String]
    private let servers: [GatewayNode]
    private let hopType: HopType
    private let isSearching: Bool

    @EnvironmentObject private var gatewayManager: GatewayManager
    @State private var isHovered = false
    @State private var isExpanded: Bool
    @State private var isCountrySelected = false
    @Binding private var path: NavigationPath
    @Binding private var scrollToModel: GatewayScrollToModel
    @Binding private var entryGateway: EntryGateway
    @Binding private var exitRouter: ExitRouter
    private var infoButtonTapCompletion: (@Sendable @MainActor (GatewayNode) -> Void)?

    public init(
        country: Country,
        servers: [GatewayNode],
        type: HopType,
        path: Binding<NavigationPath>,
        scrollToModel: Binding<GatewayScrollToModel>,
        entryGateway: Binding<EntryGateway>,
        exitRouter: Binding<ExitRouter>,
        infoButtonTapCompletion: (@Sendable @MainActor (GatewayNode) -> Void)?,
        isSearching: Bool = false
    ) {
        self.country = country
        self.servers = servers
        self.hopType = type
        self.isSearching = isSearching
        self.infoButtonTapCompletion = infoButtonTapCompletion
        self.regions = Array(Set(servers.compactMap { $0.location?.region })).sorted()
        _path = path
        _scrollToModel = scrollToModel
        _entryGateway = entryGateway
        _exitRouter = exitRouter

        let unwrappedScrollToModel = scrollToModel.wrappedValue
        let selectedServer = servers.first { $0.id == unwrappedScrollToModel.serverId }
        let shouldExpand = unwrappedScrollToModel.countryCode == country.code
        || selectedServer?.location?.twoLetterIsoCountryCode == country.code
        _isExpanded = State(initialValue: shouldExpand)
        let shouldSelect = unwrappedScrollToModel.countryCode == country.code && unwrappedScrollToModel.isCountry
        _isCountrySelected = State(initialValue: shouldSelect)
    }

    public var body: some View {
        VStack(spacing: 0) {
            countryCell()
                .id(GatewayScrollToModel.country(code: country.code).scrollToIdentifier)
            if isExpanded {
                if !regions.isEmpty && country.code == "US" {
                    ForEach(regions, id: \.self) { region in
                        Spacer()
                            .frame(height: 6)
                        GatewaysRegionCell(
                            hopType: hopType,
                            country: country,
                            region: region,
                            servers: servers.filter { $0.location?.region == region },
                            infoButtonTapCompletion: infoButtonTapCompletion,
                            path: $path,
                            entryGateway: $entryGateway,
                            exitRouter: $exitRouter,
                            scrollToModel: $scrollToModel
                        )
                        .id(GatewayScrollToModel.region(countryCode: country.code, region: region).scrollToIdentifier)
                    }
                } else {
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
    }
}

private extension GatewayCountryDropDown {
    @ViewBuilder
    func countryCell() -> some View {
        HStack(spacing: 0) {
            HStack(spacing: 0) {
                isSelectedMarker()
                FlagImage(countryCode: country.code)
                    .padding(EdgeInsets(top: 0, leading: isCountrySelected ? 12 : 16, bottom: 0, trailing: 16))
                VStack(alignment: .leading, spacing: 0) {
                    countryNameTitle()
                    serverCountNumberSubtitle()
                }
                Spacer()
            }
            .accessibilityElement(children: .combine)
            .accessibilityLabel("\(country.name) \(servers.count) \("servers".localizedString)")
            .accessibilityValue(isCountrySelected ? "selected".localizedString : "")
            .accessibilityAddTraits([.isButton])
            .contentShape(Rectangle())
            .onTapGesture {
                countryTapAction()
            }
            .accessibilityAction {
                countryTapAction()
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
                expandDidTap()
            }
            .accessibilityAction {
                expandDidTap()
            }
        }
        .onHover { newValue in
            if newValue != isHovered { isHovered = newValue }
        }
        .background {
            isHovered ? NymColor.elevationHover : NymColor.elevation
        }
    }

    @ViewBuilder
    func isSelectedMarker() -> some View {
        if isCountrySelected {
            SelectionMarker()
        }
    }

    func countryNameTitle() -> some View {
        Text(country.name)
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

private extension GatewayCountryDropDown {
    func countryTapAction() {
        ImpactGenerator.shared.softImpact()
        switch hopType {
        case .entry:
            entryGateway = .country(country.code)
        case .exit:
            exitRouter = .country(country.code)
        }
        path = .init()
    }

    func expandDidTap() {
        ImpactGenerator.shared.softImpact()
        isExpanded.toggle()
    }
}
