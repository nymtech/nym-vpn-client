import SwiftUI
import Constants
import CountriesManagerTypes
import ExternalLinkManager
import Settings
import Theme
import UIComponents

public struct GatewaysView: View {
    @ObservedObject private var viewModel: GatewaysViewModel
    @FocusState private var isSearchFocused: Bool

    public init(viewModel: GatewaysViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            optionalQuicMessage()
            Spacer()
                .frame(height: 24)
            searchView()
                .frame(maxWidth: MagicNumbers.maxWidth)
            Spacer()
                .frame(height: 24)

            if viewModel.isRefreshing {
                refreshingView()
            } else {
                mainListView()
            }
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .overlay {
            if viewModel.isGeolocationModalDisplayed {
                LocationInfoView(type: viewModel.type, isDisplayed: $viewModel.isGeolocationModalDisplayed)
                    .transition(.opacity)
                    .animation(.easeInOut, value: viewModel.isGeolocationModalDisplayed)
            }
        }
        .overlay {
            if viewModel.isServerListRefreshFailedModalDisplayed {
                RefreshErrorView(
                    isDisplayed: $viewModel.isServerListRefreshFailedModalDisplayed,
                    refresh: {
                        Task {
                            await viewModel.refreshServersList()
                        }
                    }
                )
                .transition(.opacity)
                .animation(.easeInOut, value: viewModel.isServerListRefreshFailedModalDisplayed)
            }
        }
        .onTapGesture {
            isSearchFocused = false
        }
        .onAppear {
            isSearchFocused = true
        }
    }
}

private extension GatewaysView {
    func navbar() -> some View {
        #if os(macOS)
        CustomNavBar(
            title: viewModel.type.selectHopLocalizedTitle,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateHome() }),
            rightButtons: [
                CustomNavBarButton(
                    type: .refresh,
                    action: {
                        Task {
                            await viewModel.refreshServersList()
                        }
                    }
                ),
                CustomNavBarButton(type: .info, action: { viewModel.displayInfoTooltip() })
            ]
        )
        #else
        CustomNavBar(
            title: viewModel.type.selectHopLocalizedTitle,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateHome() }),
            rightButtons: [CustomNavBarButton(type: .info, action: { viewModel.displayInfoTooltip() })]
        )
        #endif
    }

    @ViewBuilder
    func optionalQuicMessage() -> some View {
        if viewModel.shouldShowQuic {
            Spacer()
                .frame(height: 24)
            VStack(alignment: .leading, spacing: 0) {
                HStack(spacing: 0) {
                    Text(quicText())
                        .foregroundStyle(NymColor.gray1)
                        .textStyle(.Body.Small.regular)
                }
            }
            .environment(\.openURL, OpenURLAction { url in
                if url == URL(string: "app://enable-quic") {
                    viewModel.path.append(HomeLink.settings)
                    viewModel.path.append(SettingLink.censorship)
                    return .handled
                }
                return .systemAction
            })
        }
    }

    func quicText() -> AttributedString {
        let first = AttributedString("gatewaysView.quic1".localizedString)
        var secondAttr = AttributedString("gatewaysView.quic2".localizedString)
        secondAttr.underlineStyle = .single
        secondAttr.foregroundColor = NymColor.primary
        secondAttr.link = URL(string: "app://enable-quic")
        return first + AttributedString(" ") + secondAttr
    }

    func searchView() -> some View {
        SearchView(searchText: $viewModel.searchText, isSearchFocused: $isSearchFocused)
            .padding(.horizontal, 16)
    }

    func refreshingView() -> some View {
        VStack {
            GenericImage(imageName: "refresh")
                .frame(width: 24, height: 24)

            VStack(spacing: 24) {
                Text("gatewaysView.refreshingServerList.title".localizedString)
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Body.Large.regular)

                Text("gatewaysView.refreshingServerList.description".localizedString)
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Body.Large.regular)
                    .multilineTextAlignment(.center)
            }

            Spacer()
        }
        .frame(maxWidth: MagicNumbers.maxWidth)
        .padding(EdgeInsets(top: 24, leading: 16, bottom: 24, trailing: 16))
    }

    func mainListView() -> some View {
        ScrollViewReader { proxy in
            ScrollView {
                countriesGatewaysList()
                noSearchResultsView()
                foundCountriesList()
                foundRegionsList()
                foundGatewaysList()
            }
            .scrollDismissesKeyboard(.immediately)
            .scrollIndicators(.hidden)
            .frame(maxWidth: MagicNumbers.maxWidth)
            .ignoresSafeArea(.all)
            .onReceive(viewModel.$shouldScroll.filter { $0 }) { _ in
                Task { @MainActor in
                    await Task.yield() // let SwiftUI lay out with the new data
                    withAnimation {
                        proxy.scrollTo(viewModel.scrollToModel.scrollToIdentifier, anchor: .top)
                    }
                    viewModel.shouldScroll = false
                }
            }
            .refreshable {
                Task {
                    await viewModel.refreshServersList()
                }
            }
        }
    }

    @ViewBuilder
    func countriesGatewaysList() -> some View {
        if viewModel.searchText.count < viewModel.minimumSearchSymbols {
            ForEach(viewModel.countries, id: \.name) { country in
                let servers = viewModel.gatewaysInCountry(with: country.code)
                if !servers.isEmpty {
                    GatewayCountryCell(
                        country: country,
                        servers: servers,
                        type: viewModel.type,
                        path: $viewModel.path,
                        scrollToModel: $viewModel.scrollToModel,
                        entryGateway: $viewModel.connectionManager.entryGateway,
                        exitRouter: $viewModel.connectionManager.exitRouter,
                        infoButtonTapCompletion: { gateway in
                            viewModel.path.append(HomeLink.gatewayDetails(gateway: gateway, hopType: viewModel.type))
                        }
                    )
                }
            }
        }
    }

    @ViewBuilder
    func noSearchResultsView() -> some View {
        if viewModel.searchText.count >= viewModel.minimumSearchSymbols,
           viewModel.foundGateways.isEmpty,
           viewModel.foundCountries.isEmpty {
            VStack(alignment: .leading, spacing: 0) {
                Text("search.noResults".localizedString)
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Body.Large.regular)
                Spacer()
                    .frame(height: 16)
                Text("search.noResultsSubtitle".localizedString)
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Large.regular)
                Spacer()
                    .frame(height: 8)

                contactUsForHelpLinkView()
                Spacer()
                    .frame(height: 4)
            }
            .padding(.horizontal, 16)
        }
    }

    @ViewBuilder
    func contactUsForHelpLinkView() -> some View {
        if let attributtedText = contactUsForHelpAttributedString() {
            Text(attributtedText)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Large.regular)
        }
    }

    func contactUsForHelpAttributedString() -> AttributedString? {
        guard let newSupportRequestURL = URL(string: Constants.newSupportRequest.rawValue),
              let operatorURL = URL(string: Constants.operatorDocs.rawValue)
        else {
            return nil
        }

        let contactUs = "search.contactUsForHelp".localizedString
        let orText = "search.or".localizedString
        let howToRun = "search.howToRunGateway".localizedString
        let markdown = "[\(contactUs)](\(newSupportRequestURL.absoluteString)) \(orText) [\(howToRun)](\(operatorURL.absoluteString))."

        let options = AttributedString.MarkdownParsingOptions(
            interpretedSyntax: .inlineOnlyPreservingWhitespace
        )

        guard var attributed = try? AttributedString(markdown: markdown, options: options)
        else {
            return nil
        }

        for run in attributed.runs where run.link != nil {
            attributed[run.range].underlineStyle = .single
            attributed[run.range].foregroundColor = NymColor.gray1
        }
        return attributed
    }

    @ViewBuilder
    func foundCountriesList() -> some View {
        ForEach(viewModel.foundCountries, id: \.name) { country in
            let servers = viewModel.gatewaysInCountry(with: country.code)
            if !servers.isEmpty {
                GatewayCountryCell(
                    country: country,
                    servers: servers,
                    type: viewModel.type,
                    path: $viewModel.path,
                    scrollToModel: $viewModel.scrollToModel,
                    entryGateway: $viewModel.connectionManager.entryGateway,
                    exitRouter: $viewModel.connectionManager.exitRouter,
                    infoButtonTapCompletion: { gateway in
                        viewModel.path.append(HomeLink.gatewayDetails(gateway: gateway, hopType: viewModel.type))
                    },
                    isSearching: true
                )
            }
        }
        Spacer()
            .frame(height: 24)
    }

    func foundGatewaysList() -> some View {
        ForEach(viewModel.foundGateways, id: \.id) { server in
            GatewayCell(
                server: server,
                type: viewModel.type,
                path: $viewModel.path,
                scrollToModel: .constant(.empty),
                isSearching: true,
                infoButtonTapCompletion: { gateway in
                    viewModel.path.append(HomeLink.gatewayDetails(gateway: gateway, hopType: viewModel.type))
                }
            )
        }
    }

    @ViewBuilder
    func foundRegionsList() -> some View {
        ForEach(viewModel.foundRegions, id: \.region) { (country: NymCountry, region: String) in
            let servers = viewModel.gateways.filter { $0.location?.region == region }
            if !servers.isEmpty {
                GatewayRegionCell(
                    hopType: viewModel.type,
                    country: country,
                    region: region,
                    servers: servers,
                    infoButtonTapCompletion: { _ in },
                    path: $viewModel.path,
                    entryGateway: $viewModel.connectionManager.entryGateway,
                    exitRouter: $viewModel.connectionManager.exitRouter,
                    scrollToModel: .constant(.empty)
                )
            }
        }
    }
}
