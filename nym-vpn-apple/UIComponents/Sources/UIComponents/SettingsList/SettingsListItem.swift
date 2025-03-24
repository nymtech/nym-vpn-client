import SwiftUI
import AppSettings
import Theme

public struct SettingsListItem: View {
    private let viewModel: SettingsListItemViewModel

    @State private var isHovered = false

    public init(viewModel: SettingsListItemViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack(spacing: 0) {
            Spacer()
            HStack {
                iconImage()
                    .padding(.leading, 16)

                titleSubtitle()
                Spacer()

                optionalAccessoryImage()
                optionalToggleView()
            }
            Spacer()
            optionalDivider()
        }
        .frame(maxWidth: .infinity, minHeight: 64, maxHeight: 64)
        .background {
            NymColor.navigationBarBackground.opacity(isHovered ? 0.7 : 1)
        }
        .clipShape(
            .rect(
                topLeadingRadius: viewModel.topRadius,
                bottomLeadingRadius: viewModel.bottomRadius,
                bottomTrailingRadius: viewModel.bottomRadius,
                topTrailingRadius: viewModel.topRadius
            )
        )
        .padding(.horizontal, 16)
        .onTapGesture {
            viewModel.action()
        }
        .onHover { newValue in
            isHovered = newValue
        }
    }
}

private extension SettingsListItem {
    @ViewBuilder
    func optionalDivider() -> some View {
        if !viewModel.position.isLast {
            Divider()
                .frame(height: 1)
                .overlay(NymColor.settingsSeparator)
        }
    }

    @ViewBuilder
    func iconImage() -> some View {
        if let imageName = viewModel.imageName {
            Image(imageName, bundle: .module)
                .renderingMode(.template)
                .foregroundStyle(NymColor.sysOnSurface)
                .padding(.leading, 8)
        }
    }

    @ViewBuilder
    func titleSubtitle() -> some View {
        VStack(alignment: .leading) {
            Text(viewModel.title)
                .foregroundStyle(NymColor.sysOnSurface)
                .textStyle(.BodyLegacy.Large.semibold)
            if let subtitle = viewModel.subtitle {
                BouncingMarqueeTextView(
                    text: subtitle,
                    textStyle: .BodyLegacy.Medium.regular,
                    fontColor: NymColor.sysOutline,
                    speed: 70,
                    pauseDuration: 1.0
                )
            }
        }
        .clipped()
        .padding(.leading, 16)
    }

    @ViewBuilder
    func optionalAccessoryImage() -> some View {
        if let imageName = viewModel.accessory.imageName {
            Image(imageName, bundle: .module)
                .foregroundStyle(NymColor.sysOnSurface)
                .padding(.trailing, 24)
        }
    }

    @ViewBuilder
    func optionalToggleView() -> some View {
        if case let .toggle(viewModel: viewModel) = viewModel.accessory {
            ToggleView(viewModel: viewModel)
                .padding(.trailing, 24)
        }
    }
}
