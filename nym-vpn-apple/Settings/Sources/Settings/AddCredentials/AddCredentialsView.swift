import SwiftUI
import CredentialsManager
#if os(iOS)
import KeyboardManager
#endif
import Modifiers
import Theme
import UIComponents

struct AddCredentialsView: View {
#if os(iOS)
    @EnvironmentObject private var keyboardManager: KeyboardManager
#endif
    @StateObject private var viewModel: AddCredentialsViewModel
    @FocusState private var isFocused: Bool

    init(viewModel: AddCredentialsViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    var body: some View {
        VStack {
            navbar()
            GeometryReader { geometry in
#if os(iOS)
                KeyboardHostView {
                    scrollViewContent(geometry: geometry)
                }
#elseif os(macOS)
                scrollViewContent(geometry: geometry)
#endif
            }
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .onAppear {
            isFocused = true
        }
    }
}

private extension AddCredentialsView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: "",
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func scrollViewContent(geometry: GeometryProxy) -> some View {
        ScrollView {
            VStack {
                content(safeAreaInsets: geometry.safeAreaInsets)
            }
            .frame(width: geometry.size.width, height: geometry.size.height)
        }
        .onTapGesture {
            isFocused = false
        }
    }

    @ViewBuilder
    func content(safeAreaInsets: EdgeInsets) -> some View {
            Spacer()
            getStartedSection()
                .onTapGesture {
                    isFocused = false
                }

            inputView()
                .onTapGesture {
                    guard !isFocused else { return }
                    isFocused = true
                }
            if !viewModel.errorMessageTitle.isEmpty {
                errorMessageView(title: viewModel.errorMessageTitle)
            }
            Spacer()
                .frame(height: 8)

            addCredentialButton()
            Spacer()
                .frame(height: viewModel.appSettings.isSmallScreen ? 24 : 40)
    }

    @ViewBuilder
    func getStartedSection() -> some View {
        GenericImage(imageName: viewModel.logoImageName)
            .frame(width: 80, height: 80)
        Spacer()
            .frame(height: 40)

        welcomeText()
        Spacer()
            .frame(height: 16)

        getStartedTitleText()
        Spacer()
            .frame(height: 16)

        getStartedSubtitleText()
        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func welcomeText() -> some View {
        Text(viewModel.welcomeTitle)
            .textStyle(.Headline.Small.primary)
    }

    @ViewBuilder
    func getStartedTitleText() -> some View {
        Text(viewModel.getStartedTitle)
            .textStyle(.Body.Large.regular)
            .foregroundStyle(NymColor.credetnialsTitle)
    }

    @ViewBuilder
    func getStartedSubtitleText() -> some View {
        Text(viewModel.getStartedSubtitle)
            .textStyle(.Body.Small.primary)
            .foregroundStyle(NymColor.credetnialsSubtitle)
            .multilineTextAlignment(.center)
            .padding(.horizontal, 16)
    }

    @ViewBuilder
    func inputView() -> some View {
        LazyVStack(alignment: .leading) {
            TextField(viewModel.credentialsPlaceholderTitle, text: $viewModel.credentialText, axis: .vertical)
                .textStyle(NymTextStyle.Body.Large.regular)
                .padding(16)
                .lineLimit(8, reservesSpace: true)
                .focused($isFocused)
                .textFieldStyle(PlainTextFieldStyle())
            Spacer()
        }
        .contentShape(
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
        )
        .frame(height: 212)
        .cornerRadius(8)
        .overlay {
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
                .stroke(viewModel.textFieldStrokeColor, lineWidth: 1)
        }
        .overlay(alignment: .topLeading) {
            Text(viewModel.credentialSubtitle)
                .foregroundStyle(viewModel.credentialSubtitleColor)
                .textStyle(.Body.Small.primary)
                .padding(4)
                .background(NymColor.background)
                .position(x: 50, y: 0)
        }
        .padding(EdgeInsets(top: 12, leading: 16, bottom: viewModel.bottomPadding, trailing: 16))
    }

    @ViewBuilder
    func errorMessageView(title: String) -> some View {
        HStack {
            Text(title)
                .foregroundStyle(NymColor.sysError)
                .textStyle(NymTextStyle.Body.Small.primary)
            Spacer()
        }
        .padding(EdgeInsets(top: 0, leading: 28, bottom: 16, trailing: 28))
    }

    @ViewBuilder
    func addCredentialButton() -> some View {
        GenericButton(title: viewModel.addCredentialButtonTitle)
            .padding(.horizontal, 16)
            .onTapGesture {
                viewModel.importCredentials()
                isFocused = false
            }
    }
}
