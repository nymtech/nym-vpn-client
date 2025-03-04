import SwiftUI
import ConnectionManager
import CountriesManager
import CountriesManagerTypes
import Theme

public struct GatewayCountryDropDown: View {
    private let country: Country
    private let servers: [GatewayNode]
    private let hopType: HopType
    private let isSearching: Bool

    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var countriesManager: CountriesManager
    @State private var isHovered = false
    @State private var isExpanded = false
    @Binding private var path: NavigationPath
    @Binding private var isServerModalDisplayed: Bool
    @Binding private var serverInfoModalServer: GatewayNode?
    @Binding private var scrollToServer: GatewayNode?

    public init(
        country: Country,
        servers: [GatewayNode],
        type: HopType,
        path: Binding<NavigationPath>,
        isServerModalDisplayed: Binding<Bool>,
        serverInfoModalServer: Binding<GatewayNode?>,
        scrollToServer: Binding<GatewayNode?>,
        isSearching: Bool = false
    ) {
        self.country = country
        self.servers = servers
        self.hopType = type
        self.isSearching = isSearching
        _path = path
        _isServerModalDisplayed = isServerModalDisplayed
        _serverInfoModalServer = serverInfoModalServer
        _scrollToServer = scrollToServer
    }

    public var body: some View {
        VStack(spacing: 0) {
            countryCell()
            if isExpanded {
                ForEach(servers, id: \.id) { server in
                    GatewayCell(
                        server: server,
                        type: hopType,
                        path: $path,
                        isServerModalDisplayed: $isServerModalDisplayed,
                        serverInfoModalServer: $serverInfoModalServer
                    )
                    .id(server.id)
                }
            }
        }
        .animation(.easeInOut, value: isExpanded)
        .onAppear {
            guard let server = selectedServer() else { return }
            isExpanded = true
            scrollToServer = server
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
                    .padding(EdgeInsets(top: 0, leading: isCountrySelected() ? 12 : 16, bottom: 0, trailing: 16))
                VStack(alignment: .leading, spacing: 0) {
                    countryNameTitle()
                    serverCountNumberSubtitle()
                }
                Spacer()
            }
            .contentShape(Rectangle())
            .onTapGesture {
                switch hopType {
                case .entry:
                    connectionManager.entryGateway = .country(country)
                case .exit:
                    connectionManager.exitRouter = .country(country)
                }
                path = .init()
            }
            HStack(spacing: 0) {
                lineSeparator()
                arrowDropDown()
            }
            .contentShape(Rectangle())
            .onTapGesture {
                isExpanded.toggle()
            }
        }
        .onHover { newValue in
            isHovered = newValue
        }
        .background {
            isHovered ? NymColor.elevationHover : NymColor.elevation
        }
    }

    @ViewBuilder
    func isSelectedMarker() -> some View {
        if isCountrySelected() {
            SelectionMarker()
        }
    }

    func countryNameTitle() -> some View {
        Text(country.name)
            .foregroundStyle(NymColor.primary)
            .textStyle(.BodyLegacy.Large.regular)
    }

    func serverCountNumberSubtitle() -> some View {
        Text("\(servers.count) \("servers".localizedString)")
            .foregroundStyle(NymColor.gray1)
            .textStyle(.BodyLegacy.Small.primary)
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
    func isCountrySelected() -> Bool {
        switch hopType {
        case .entry:
            connectionManager.entryGateway.countryCode == country.code && connectionManager.entryGateway.isCountry
        case .exit:
            connectionManager.exitRouter.countryCode == country.code && connectionManager.entryGateway.isCountry
        }
    }

    func selectedServer() -> GatewayNode? {
        guard connectionManager.entryGateway.isGateway else { return nil }
        switch hopType {
        case .entry:
            return servers.first { $0.id == connectionManager.entryGateway.gatewayId }
        case .exit:
            return servers.first { $0.id == connectionManager.exitRouter.gatewayId }
        }
    }
}
