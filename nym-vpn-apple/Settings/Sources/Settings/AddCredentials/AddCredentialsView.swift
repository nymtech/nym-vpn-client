import SwiftUI
import CredentialsManager
import KeyboardManager
import Modifiers
import Theme
import UIComponents

struct AddCredentialsView: View {
    @EnvironmentObject private var keyboardManager: KeyboardManager
    @StateObject private var viewModel: AddCredentialsViewModel
    @FocusState private var isFocused: Bool

    init(viewModel: AddCredentialsViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    var body: some View {
        VStack {
            navbar()
            GeometryReader { geometry in
                KeyboardHostView {
                    ScrollView {
                        VStack {
                            content()
                        }
                        .frame(width: geometry.size.width, height: geometry.size.height )
                    }

                    .onTapGesture {
                        isFocused = false
                    }
                }
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
    func content() -> some View {
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
                .frame(height: 24)
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
            .textStyle(.Body.Large.primary)
    }

    @ViewBuilder
    func getStartedSubtitleText() -> some View {
        Text(viewModel.getStartedSubtitle)
            .textStyle(.Body.Small.primary)
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
                .foregroundStyle(NymColor.sysOnSurface)
                .textStyle(.Body.Small.primary)
                .padding(4)
                .background(NymColor.background)
                .position(x: 50, y: 0)
        }
        .padding(EdgeInsets(top: 12, leading: 12, bottom: viewModel.bottomPadding, trailing: 12))
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
