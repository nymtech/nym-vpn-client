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
        VStack(alignment: .center, spacing: 0) {
            HStack(spacing: 0) {
                iconImage()
                    .padding(.leading, 16)
                titleSubtitle()
                    .padding(.horizontal, 16)
                Spacer()
                HStack(spacing: 0) {
                    optionalAccessoryImage()
                    optionalToggleView()
                }
            }
            .frame(height: 64)
            optionalMultilineLabel()
            optionalDivider()
        }
        .background {
            NymColor.elevation.opacity(isHovered ? 0.7 : 1)
        }
        .clipShape(
            .rect(
                topLeadingRadius: viewModel.topRadius,
                bottomLeadingRadius: viewModel.bottomRadius,
                bottomTrailingRadius: viewModel.bottomRadius,
                topTrailingRadius: viewModel.topRadius
            )
        )
        .onTapGesture {
            viewModel.action()
        }
        .onHover { newValue in
            isHovered = newValue
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(viewModel.title) \(viewModel.subtitle ?? "")")
        .accessibilityValue(viewModel.accessory.accessibilityValue)
        .accessibilityHint(viewModel.accessory.accessibilityHint)
        .accessibilityAddTraits([.isButton])
    }
}

private extension SettingsListItem {
    @ViewBuilder
    func optionalDivider() -> some View {
        if !viewModel.position.isLast {
            Divider()
                .frame(height: 1)
                .overlay(NymColor.background)
        }
    }

    @ViewBuilder
    func iconImage() -> some View {
        if let imageName = viewModel.imageName {
            Image(imageName, bundle: .module)
                .renderingMode(.template)
                .foregroundStyle(NymColor.gray1)
        } else if let systemImageName = viewModel.systemImageName {
            Image(systemName: systemImageName)
                .renderingMode(.template)
                .foregroundStyle(NymColor.gray1)
                .font(.system(size: 18, weight: .bold))
        }
    }

    @ViewBuilder
    func titleSubtitle() -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(viewModel.title)
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Large.regular)

            if let subtitle = viewModel.subtitle {
                BouncingMarqueeTextView(
                    text: subtitle,
                    textStyle: .Body.Small.regular,
                    fontColor: NymColor.gray1,
                    speed: 70,
                    pauseDuration: 1.0
                )
            }
        }
    }

    @ViewBuilder
    func optionalAccessoryImage() -> some View {
        if let imageName = viewModel.accessory.imageName {
            Image(imageName, bundle: .module)
                .resizable()
                .frame(width: 24, height: 24)
                .foregroundStyle(viewModel.accessory.imageColor)
                .padding(.trailing, 16)
        }
    }

    @ViewBuilder
    func optionalToggleView() -> some View {
        if case let .toggle(viewModel: viewModel) = viewModel.accessory {
            ToggleView(viewModel: viewModel)
                .padding(.trailing, 16)
        }
    }

    @ViewBuilder
    func optionalMultilineLabel() -> some View {
        if let multilineText = viewModel.multilineText {
            HStack(spacing: 0) {
                Text(multilineText)
                    .textStyle(.Body.Medium.regular)
                    .foregroundStyle(NymColor.gray1)
                    .padding(.horizontal, 16)
                    .tint(NymColor.gray1)
                Spacer()
            }
            Spacer()
                .frame(height: 18)
        }
    }
}
