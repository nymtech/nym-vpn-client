import SwiftUI
import Theme
import UIComponents

struct CredentialView: View {
    @ObservedObject private var viewModel: CredentialViewModel

    init(viewModel: CredentialViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        VStack {
            timeLeftText()
            timeUsedProgressIndicator()
            extendCredentialSection()
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }
}

private extension CredentialView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func timeLeftText() -> some View {
        HStack {
            Text(viewModel.daysLeftLocalizedString())
                .textStyle(.Label.Huge.bold)
                .foregroundStyle(NymColor.sysOnSurface)
                .padding(.horizontal, 16)

            Spacer()
        }
    }

    @ViewBuilder
    func timeUsedProgressIndicator() -> some View {
        ProgressView(value: viewModel.timeUsed)
            .tint(NymColor.primaryOrange)
            .padding(16)
    }

    @ViewBuilder
    func extendCredentialSection() -> some View {
        if viewModel.displayExtendSection {
            HStack {
                Text(viewModel.soonToExpireLocalizedString)
                    .textStyle(.Body.Large.regular)
                    .padding(EdgeInsets(top: 8, leading: 16, bottom: 8, trailing: 16))

                GenericButton(title: viewModel.addNewCredentialLocalizedString)
                    .frame(height: 20)
                    .padding(.trailing, 16)
                    .onTapGesture {
                        viewModel.navigateToAddCredential()
                    }
            }
        }
    }
}
