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
                ScrollView {
                    VStack(spacing: 0) {
                        titleSubtitleSection()
                        sections()
                            .frame(maxWidth: MagicNumbers.maxWidth)
                    }
                    .padding(.horizontal, 16)
                }
            }
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }
}

private extension SupportView {
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    func titleSubtitleSection() -> some View {
        HStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 16) {
                Text("⚠️ \("support.protectTitle".localizedString)")
                    .textStyle(.Headline.Small.regular)
                    .foregroundStyle(NymColor.primary)

                Text(subtitleText())
                    .textStyle(.Body.Medium.regular)
                    .foregroundStyle(NymColor.gray1)
            }
            Spacer()
        }
        .padding(.vertical, 24)
    }

    func sections() -> some View {
        SettingsList<SupportSectionKind>(
            viewModel:
                SettingsListViewModel(
                    sections: viewModel.sections
                )
        )
    }
}

private extension SupportView {
    func subtitleText() -> AttributedString {
        let first = AttributedString("support.protect.subtitle1".localizedString)
        let second = AttributedString("support.protect.subtitle2".localizedString)
        return first + "\n\n" + second
    }
}
