import SwiftUI
import CountriesManagerTypes
import Device
import ExternalLinkManager
import Theme
import UIComponents

public struct GatewaysView: View {
    @StateObject private var viewModel: GatewaysViewModel
    @FocusState private var isSearchFocused: Bool

    public init(viewModel: GatewaysViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)

            searchView()
                .frame(maxWidth: Device.type == .ipad ? 358 : .infinity)
            Spacer()
                .frame(height: 24)

            ScrollViewReader { proxy in
                ScrollView {
                    countriesGatewaysList()
                    //                noSearchResultsView()
                }
                .frame(maxWidth: Device.type == .ipad ? 358 : .infinity)
                .ignoresSafeArea(.all)
                .onChange(of: viewModel.scrollToServer) { _ in
                    guard let server = viewModel.scrollToServer else { return }
                    withAnimation {
                        proxy.scrollTo(server.id, anchor: .center)
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
            }
        }
        .overlay {
            if viewModel.isServerInfoModalDisplayed, let server = viewModel.serverInfoModalServer {
                GatewayInfoModal(server: server, isDisplayed: $viewModel.isServerInfoModalDisplayed)
                    .transition(.opacity)
                    .animation(.easeInOut, value: viewModel.isServerInfoModalDisplayed)
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
        ForEach(viewModel.countries, id: \.name) { country in
            GatewayCountryDropDown(
                country: country,
                servers: viewModel.gatewaysInCountry(with: country.code),
                type: viewModel.type,
                path: $viewModel.path,
                isServerModalDisplayed: $viewModel.isServerInfoModalDisplayed,
                serverInfoModalServer: $viewModel.serverInfoModalServer,
                scrollToServer: $viewModel.scrollToServer
            )
        }
    }

//    @ViewBuilder
//    func noSearchResultsView() -> some View {
//        if !viewModel.searchText.isEmpty && viewModel.countries?.isEmpty ?? true {
//            VStack {
//                Text(viewModel.noResultsText)
//                    .textStyle(.Body.Medium.regular)
//                    .padding(.top, 96)
//                Spacer()
//            }
//        }
//    }
}
