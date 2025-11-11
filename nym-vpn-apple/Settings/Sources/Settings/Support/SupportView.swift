import SwiftUI
import Device
import Theme
import UIComponents

struct SupportView: View {
    @StateObject private var viewModel: SupportViewModel

    init(viewModel: SupportViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    var body: some View {
        VStack(spacing: 0) {
            navbar()
            VStack(spacing: 0) {
                Spacer()
                    .frame(height: 24)
                ScrollView {
                    sections()
                        .frame(maxWidth: MagicNumbers.maxWidth)
                }
            }
            .padding(.horizontal, 16)
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .overlay {
            if viewModel.isResetVPNProfileDisplayed {
                ResetVPNProfileDialog(
                    viewModel: ResetVPNProfileDialogViewModel(
                        isDisplayed: $viewModel.isResetVPNProfileDisplayed,
                        impactGenerator: .shared,
                        action: {
                            viewModel.resetVPNProfile()
                        }
                    )
                )
            }
        }
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }
}

private extension SupportView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func sections() -> some View {
        ForEach(viewModel.sections, id: \.self) { viewModel in
            SettingsListItem(viewModel: viewModel)
            Spacer()
                .frame(height: 24)
        }
    }
}
