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
        VStack {
            navbar()
            Spacer()
                .frame(height: 24)
            sections()
                .frame(maxWidth: Device.type == .ipad ? 358 : 390)
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
