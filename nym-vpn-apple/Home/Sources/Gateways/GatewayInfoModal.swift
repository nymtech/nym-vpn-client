import SwiftUI
import ConnectionManager
import CountriesManager
import CountriesManagerTypes
import Theme
import UIComponents
#if os(iOS)
import ImpactGenerator
#endif

public struct GatewayInfoModal: View {
    private let server: GatewayNode

    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var countriesManager: CountriesManager
    @Binding private var isDisplayed: Bool

    public init(server: GatewayNode, isDisplayed: Binding<Bool>) {
        self.server = server
        _isDisplayed = isDisplayed
    }

    public var body: some View {
        ZStack {
            Rectangle()
                .foregroundColor(.black)
                .opacity(0.3)
                .background(Color.clear)
                .contentShape(Rectangle())

            HStack {
                Spacer()
                    .frame(width: 24)

                VStack(alignment: .leading) {
                    Spacer()
                        .frame(height: 24)

                    serverTitle()
                    Spacer()
                        .frame(height: 12)

                    serverDetails()
                    Spacer()
                        .frame(height: 24)

                    identityKeyContainer()
                    Spacer()
                        .frame(height: 24)

                    closeButton()
                    Spacer()
                        .frame(height: 24)
                }
                .background(NymColor.elevation)
                .cornerRadius(16)

                Spacer()
                    .frame(width: 24)
            }
        }
        .edgesIgnoringSafeArea(.all)
    }
}

private extension GatewayInfoModal {
    func serverTitle() -> some View {
        Text(server.moniker ?? server.id)
            .foregroundStyle(NymColor.primary)
            .textStyle(.Headline.Medium.regular)
            .padding(.horizontal, 24)
    }

    func serverDetails() -> some View {
        HStack {
            scoreImage()
            Spacer()
                .frame(width: 8)

            lineSeparator()
            Spacer()
                .frame(width: 8)

            FlagImage(countryCode: server.countryCode, width: 16, height: 16)
            Spacer()
                .frame(width: 8)

            countryNameText()
        }
        .padding(.horizontal, 24)
    }

    func scoreImage() -> some View {
        GenericImage(imageName: scoreImageName())
            .frame(width: 16, height: 16)
            .padding(0)
    }

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

    func lineSeparator() -> some View {
        Rectangle()
            .foregroundColor(NymColor.gray2)
            .frame(width: 1, height: 24)
            .padding(0)
    }

    @ViewBuilder
    func countryNameText() -> some View {
        if let country = countriesManager.country(with: server.countryCode) {
            Text(country.name)
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Large.regular)
                .padding(0)
        }
    }

    func identityKeyContainer() -> some View {
        VStack(alignment: .leading) {
            identityKeyTitle()
            Spacer()
                .frame(height: 8)

            HStack(alignment: .top) {
                identityKeyValueText()
                Spacer()
                    .frame(width: 16)
                GenericImage(imageName: "copy", allowsHover: true)
                    .foregroundStyle(NymColor.accent)
                    .contentShape(Rectangle())
                    .frame(width: 16, height: 16)
                    .onTapGesture {
                        copyToPasteboard()
                    }
            }
        }
        .padding(.horizontal, 24)
    }

    func copyToPasteboard() {
#if os(iOS)
        UIPasteboard.general.string = server.id
        ImpactGenerator.shared.impact()
#elseif os(macOS)
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(server.id, forType: .string)
#endif
    }

    func identityKeyTitle() -> some View {
        Text("getewaySelector.identityKey".localizedString)
            .foregroundStyle(NymColor.gray1)
            .textStyle(.Body.Small.regular)
    }

    func identityKeyValueText() -> some View {
        Text(server.id)
            .foregroundStyle(NymColor.primary)
            .textStyle(.Body.Medium.regular)
    }

    func closeButton() -> some View {
        GenericButton(title: "getewaySelector.close".localizedString)
            .padding(.horizontal, 24)
            .onTapGesture {
                isDisplayed.toggle()
            }
    }
}
