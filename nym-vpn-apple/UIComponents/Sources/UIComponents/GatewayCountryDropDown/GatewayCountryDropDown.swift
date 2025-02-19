import SwiftUI
import ConnectionManager
import CountriesManagerTypes
import Theme

public struct GatewayCountryDropDown: View {
    private let country: Country
    private let servers: [GatewayNode]
    private let isSelected: Bool
    private let hopType: HopType

    @EnvironmentObject private var connectionManager: ConnectionManager
    @State private var isExpanded = false
    @Binding private var path: NavigationPath

    public init(
        country: Country,
        servers: [GatewayNode],
        isSelected: Bool,
        type: HopType,
        path: Binding<NavigationPath>
    ) {
        self.country = country
        self.servers = servers
        self.isSelected = isSelected
        self.hopType = type
        _path = path
    }

    public var body: some View {
        VStack(spacing: 0) {
            countryCell()
            if isExpanded {
                ForEach(servers, id: \.id) { server in
                    GatewayCell(server: server, type: hopType, path: $path)
                }
            }
        }
        .animation(.easeInOut, value: isExpanded)
    }
}

private extension GatewayCountryDropDown {
    @ViewBuilder
    func countryCell() -> some View {
        HStack(spacing: 0) {
            HStack(spacing: 0) {
                isSelectedMarker()
                FlagImage(countryCode: country.code)
                    .padding(16)
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
        .background {
            NymColor.elevation
                .ignoresSafeArea()
        }
    }

    @ViewBuilder
    func isSelectedMarker() -> some View {
        if isSelected {
            UnevenRoundedRectangle(bottomTrailingRadius: 4, topTrailingRadius: 4)
                .foregroundColor(NymColor.accent)
                .frame(width: 4)
        }
    }

    func countryNameTitle() -> some View {
        Text(country.name)
            .foregroundStyle(NymColor.primary)
            .textStyle(.BodyLegacy.Large.regular)
    }

    func serverCountNumberSubtitle() -> some View {
        Text("\(servers.count) " + "servers".localizedString)
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
