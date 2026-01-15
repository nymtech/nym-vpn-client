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
                KeyboardHostView(bottomSafeAreaInset: geometry.safeAreaInsets.bottom) {
                    scrollViewContent(geometry: geometry)
                }
#elseif os(macOS)
                scrollViewContent(geometry: geometry)
#endif
            }
            .frame(maxWidth: MagicNumbers.maxWidth)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
#if os(macOS)
        .ignoresSafeArea(edges: [.bottom])
#endif
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
        .onChange(of: isFocused) { _, newValue in
            viewModel.isFocused = newValue
        }
    }
}

private extension AddCredentialsView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            useElevationBackground: true,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func scrollViewContent(geometry: GeometryProxy) -> some View {
        ScrollView {
            VStack(spacing: 0) {
                content(safeAreaInsets: geometry.safeAreaInsets)
                    .padding(.horizontal, 24)
            }
            .frame(maxWidth: .infinity)
            .frame(minHeight: geometry.size.height, alignment: .top)
        }
        .scrollIndicators(.hidden)
        .scrollDismissesKeyboard(.interactively)
        .onTapGesture { isFocused = false }
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
        TermsAndConditionsView()
        Spacer()
            .frame(height: 24)
    }

    @ViewBuilder
    func getStartedSection() -> some View {
        loginTitle()
        Spacer()
        enterPassphraseTitleText()
        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func loginTitle() -> some View {
        HStack {
            Spacer()
            Text("login".localizedString)
                .textStyle(.Headline.Large.regular)
            Spacer()
        }
    }

    @ViewBuilder
    func enterPassphraseTitleText() -> some View {
        HStack {
            Text("addCredentials.getStarted.Title".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
                .multilineTextAlignment(.leading)
            Spacer()
        }
    }

    @ViewBuilder
    func inputView() -> some View {
        LazyVStack(alignment: .leading) {
            TextField("addCredentials.placeholder".localizedString, text: $viewModel.credentialText, axis: .vertical)
// https://stackoverflow.com/questions/74989806/how-to-dismiss-keyboard-in-swiftui-keyboard-when-pressing-done
//                .onSubmit {
//                    viewModel.importCredentials()
//                    isFocused = false
//                }
                .onChange(of: viewModel.credentialText) { [weak viewModel] _, _ in
                    if viewModel?.credentialText.last?.isNewline == .some(true) {
                        login()
                    }
                }
                .redacted(reason: .privacy)
                .submitLabel(.done)
                .textStyle(NymTextStyle.Body.Large.regular)
                .padding(16)
                .lineLimit(4, reservesSpace: true)
                .focused($isFocused)
                .textFieldStyle(PlainTextFieldStyle())
                .autocorrectionDisabled()
            Spacer()
        }
        .contentShape(
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
        )
        .frame(height: 130)
        .cornerRadius(8)
        .overlay {
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
                .stroke(viewModel.textFieldStrokeColor, lineWidth: 1)
        }
        .overlay(alignment: .topLeading) {
            Text("addCredtenials.mnemonic".localizedString)
                .foregroundStyle(viewModel.credentialSubtitleColor)
                .textStyle(.Body.Small.regular)
                .padding(4)
                .background(NymColor.background)
                .position(x: 70, y: 0)
        }
        .padding(EdgeInsets(top: 12, leading: 0, bottom: viewModel.bottomPadding, trailing: 0))
    }

    @ViewBuilder
    func errorMessageView(title: String) -> some View {
        HStack {
            Text(title)
                .foregroundStyle(NymColor.error)
                .multilineTextAlignment(.leading)
                .lineLimit(nil)
                .textStyle(.Body.Small.regular)
            Spacer()
        }
    }

    @ViewBuilder
    func loginButton() -> some View {
        GenericButton(title: "login".localizedString)
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
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Large.regular)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 16)
                .environment(\.openURL, OpenURLAction { url in
                    if url == URL(string: viewModel.createAccounAppLink) {
                        viewModel.navigateToCreateAccount()
                        return .handled
                    }
                    return .systemAction
                })
        }
    }
}

private extension AddCredentialsView {
    func login() {
        isFocused = false
        viewModel.importCredentials()
    }
}
