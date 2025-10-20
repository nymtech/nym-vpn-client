import SwiftUI
import Constants
import CountriesManagerTypes
import ExternalLinkManager
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
            Spacer()
                .frame(height: 24)

            searchView()
                .frame(maxWidth: MagicNumbers.maxWidth)
            Spacer()
                .frame(height: 24)

            ScrollViewReader { proxy in
                ScrollView {
                    countriesGatewaysList()
                    noSearchResultsView()
                    foundCountriesList()
                    foundUSRegionsList()
                    foundGatewaysList()
                }
                .scrollDismissesKeyboard(.immediately)
                .scrollIndicators(.hidden)
                .frame(maxWidth: MagicNumbers.maxWidth)
                .ignoresSafeArea(.all)
                .onAppear {
                    withAnimation {
                        proxy.scrollTo(viewModel.scrollToModel.scrollToIdentifier, anchor: .top)
                    }
                }
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
                LocationInfoView(
                    viewModel: LocationInfoViewModel(
                        externalLinkManager: ExternalLinkManager.shared,
                        isDisplayed: $viewModel.isGeolocationModalDisplayed
                    )
                )
                .transition(.opacity)
                .animation(.easeInOut, value: viewModel.isGeolocationModalDisplayed)
            }
        }
        .onTapGesture {
            isSearchFocused = false
        }
    }
}

private extension GatewaysView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.type.selectHopLocalizedTitle,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateHome() }),
            rightButton: CustomNavBarButton(type: .info, action: { viewModel.displayInfoTooltip() })
        )
    }

    @ViewBuilder
    func searchView() -> some View {
        SearchView(searchText: $viewModel.searchText, isSearchFocused: $isSearchFocused)
            .padding(.horizontal, 16)
    }

    @ViewBuilder
    func countriesGatewaysList() -> some View {
        if viewModel.searchText.count < viewModel.minimumSearchSymbols {
            ForEach(viewModel.countries, id: \.name) { country in
                GatewayCountryDropDown(
                    country: country,
                    servers: viewModel.gatewaysInCountry(with: country.code),
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
            GatewayCountryDropDown(
                country: country,
                servers: viewModel.gatewaysInCountry(with: country.code),
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
        Spacer()
            .frame(height: 24)
    }

    @ViewBuilder
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
    func foundUSRegionsList() -> some View {
        if let usCountry = viewModel.gatewayManager.localizedCountry(with: "US") {
            ForEach(viewModel.foundUSRegions, id: \.self) { region in
                GatewaysRegionCell(
                    hopType: viewModel.type,
                    country: usCountry,
                    region: region,
                    servers: viewModel.gatewayManager.vpn.filter { $0.location?.region == region },
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
