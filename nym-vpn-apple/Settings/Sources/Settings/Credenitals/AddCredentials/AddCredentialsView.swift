import SwiftUI
import AppSettings
import CredentialsManager
import Device
#if os(iOS)
import ExternalLinkManager
import KeyboardManager
#endif
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
            .frame(maxWidth: Device.type == .ipad ? 358 : 390)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
#if os(iOS)
        .fullScreenCover(isPresented: $viewModel.isScannerDisplayed) {
            QRScannerView(
                viewModel: QRScannerViewModel(
                    isDisplayed: $viewModel.isScannerDisplayed,
                    scannedText: $viewModel.credentialText,
                    externalLinkManager: ExternalLinkManager.shared,
                    keyboardManager: KeyboardManager.shared
                )
            )
        }
#endif
//        .onAppear {
//            isFocused = viewModel.isFocused
//        }
        .onChange(of: isFocused) {
            viewModel.isFocused = $0
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
        .scrollIndicators(.hidden)
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

        HStack {
            loginButton()
// #if os(iOS)
//            qrScannerButton()
//                .padding(.trailing, 16)
// #endif
        }
        .padding(.vertical, 16)

        createAccount()

        Spacer()
            .frame(height: viewModel.appSettings.isSmallScreen ? 24 : 40)
    }

    @ViewBuilder
    func getStartedSection() -> some View {
        Spacer()
            .frame(height: 40)

        welcomeText()
        Spacer()
            .frame(height: 16)

        getStartedTitleText()
        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func welcomeText() -> some View {
        Text(viewModel.welcomeTitle)
            .textStyle(.HeadlineLegacy.Small.primary)
    }

    @ViewBuilder
    func getStartedTitleText() -> some View {
        Text(viewModel.getStartedTitle)
            .textStyle(.BodyLegacy.Large.regular)
            .foregroundStyle(NymColor.credetnialsTitle)
            .multilineTextAlignment(.center)
            .padding(.horizontal, 16)
    }

    @ViewBuilder
    func inputView() -> some View {
        LazyVStack(alignment: .leading) {
            TextField(viewModel.credentialsPlaceholderTitle, text: $viewModel.credentialText, axis: .vertical)
// https://stackoverflow.com/questions/74989806/how-to-dismiss-keyboard-in-swiftui-keyboard-when-pressing-done
//                .onSubmit {
//                    viewModel.importCredentials()
//                    isFocused = false
//                }
                .onChange(of: viewModel.credentialText) { [weak viewModel] _ in
                    if viewModel?.credentialText.last?.isNewline == .some(true) {
                        login()
                    }
                }
                .submitLabel(.done)
                .textStyle(NymTextStyle.BodyLegacy.Large.regular)
                .padding(16)
                .lineLimit(8, reservesSpace: true)
                .focused($isFocused)
                .textFieldStyle(PlainTextFieldStyle())
                .autocorrectionDisabled()
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
            Text(viewModel.mnemonicSubtitle)
                .foregroundStyle(viewModel.credentialSubtitleColor)
                .textStyle(.BodyLegacy.Small.primary)
                .padding(4)
                .background(NymColor.background)
                .position(x: 55, y: 0)
        }
        .padding(EdgeInsets(top: 12, leading: 16, bottom: viewModel.bottomPadding, trailing: 16))
    }

    @ViewBuilder
    func errorMessageView(title: String) -> some View {
        HStack {
            Text(title)
                .foregroundStyle(NymColor.sysError)
                .lineLimit(nil)
                .textStyle(NymTextStyle.BodyLegacy.Small.primary)
            Spacer()
        }
        .padding(EdgeInsets(top: 0, leading: 28, bottom: 16, trailing: 28))
    }

    @ViewBuilder
    func loginButton() -> some View {
        GenericButton(title: viewModel.loginButtonTitle)
            .padding(.horizontal, 16)
            .onTapGesture {
                login()
            }
    }

//    @ViewBuilder
//    func qrScannerButton() -> some View {
//        GenericImage(systemImageName: viewModel.scannerIconName)
//            .frame(width: 56, height: 56)
//            .foregroundStyle(NymColor.connectTitle)
//            .background(NymColor.primaryOrange)
//            .cornerRadius(8)
//            .onTapGesture {
//                Task { @MainActor in
//                    viewModel.isScannerDisplayed.toggle()
//                }
//            }
//    }

    @ViewBuilder
    func createAccount() -> some View {
        if let createAccountAttributedString = viewModel.createAnAccountAttributedString() {
            Text(createAccountAttributedString)
                .tint(NymColor.accent)
                .foregroundStyle(NymColor.sysOnSurface)
                .textStyle(.BodyLegacy.Large.regular)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 16)
        }
    }
}

private extension AddCredentialsView {
    func login() {
        viewModel.importCredentials()
        isFocused = false
    }
}
