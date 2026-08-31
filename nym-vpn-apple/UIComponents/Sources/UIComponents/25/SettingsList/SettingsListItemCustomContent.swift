import SwiftUI
import AppSettings
import Theme

public struct SettingsListItemCustomContent<CustomContent: View>: View {
    private let viewModel: SettingsListItemViewModel
    private let customContent: (() -> CustomContent)?
    private let combineAccessibilityChildren: Bool

    @State private var isHovered = false
    @State private var isToggleOn = false

    public init(
        viewModel: SettingsListItemViewModel,
        customContent: (() -> CustomContent)? = nil,
        combineAccessibilityChildren: Bool = true
    ) {
        self.viewModel = viewModel
        self.customContent = customContent
        self.combineAccessibilityChildren = combineAccessibilityChildren
        if case let .toggle(isOn, _) = viewModel.accessory {
            _isToggleOn = State(initialValue: isOn.wrappedValue)
        }
    }

    private var toggleValue: Bool {
        if case let .toggle(isOn, _) = viewModel.accessory {
            return isOn.wrappedValue
        }
        return false
    }

    public var body: some View {
        VStack(alignment: .center, spacing: 0) {
            HStack(spacing: 0) {
                if viewModel.accessory.isToggle {
                    mainContent()
                } else {
                    mainContent()
                        .contentShape(Rectangle())
                        .onTapGesture {
                            viewModel.action()
                        }
                }
                HStack(spacing: 0) {
                    optionalAccessoryImage()
                    optionalToggleView()
                }
            }
            .frame(height: 64)
            optionalMultilineLabel()
            optionalCustomContent()
            optionalDivider()
        }
        .background {
            UnevenRoundedRectangle(
                topLeadingRadius: viewModel.topRadius,
                bottomLeadingRadius: viewModel.bottomRadius,
                bottomTrailingRadius: viewModel.bottomRadius,
                topTrailingRadius: viewModel.topRadius
            )
            .fill(viewModel.type.backgroundColor.opacity(isHovered ? 0.7 : 1))
        }
        .overlay {
            UnevenRoundedRectangle(
                topLeadingRadius: viewModel.topRadius,
                bottomLeadingRadius: viewModel.bottomRadius,
                bottomTrailingRadius: viewModel.bottomRadius,
                topTrailingRadius: viewModel.topRadius
            )
            .stroke(viewModel.type.strokeColor, lineWidth: 1)
            .allowsHitTesting(false)
        }
        .clipShape(
            UnevenRoundedRectangle(
                topLeadingRadius: viewModel.topRadius,
                bottomLeadingRadius: viewModel.bottomRadius,
                bottomTrailingRadius: viewModel.bottomRadius,
                topTrailingRadius: viewModel.topRadius
            )
        )
        .onHover { newValue in
            guard !viewModel.isHoveredHighlightDisabled else { return }
            isHovered = newValue
        }
        .onChange(of: isToggleOn) { _, newValue in
            if case let .toggle(isOn, _) = viewModel.accessory, isOn.wrappedValue != newValue {
                isOn.wrappedValue = newValue
            }
        }
        .onChange(of: toggleValue) { _, newValue in
            if isToggleOn != newValue {
                isToggleOn = newValue
            }
        }
        .settingsListAccessibility(viewModel, combineChildren: combineAccessibilityChildren)
    }
}

private extension SettingsListItemCustomContent {
    @ViewBuilder
    func mainContent() -> some View {
        HStack(spacing: 0) {
            iconImage()
                .padding(.leading, 16)
            titleSubtitle()
                .padding(.horizontal, 16)
            Spacer()
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
    }

    @ViewBuilder
    func optionalDivider() -> some View {
        if !viewModel.position.isLast {
            Divider()
                .frame(height: 1)
                .overlay(Color.Nym.divider)
        }
    }

    @ViewBuilder
    func iconImage() -> some View {
        if let imageName = viewModel.imageName {
            Image(imageName, bundle: .module)
                .renderingMode(.template)
                .foregroundStyle(Color.Nym.textSecondary)
        } else if let systemImageName = viewModel.systemImageName {
            Image(systemName: systemImageName)
                .renderingMode(.template)
                .foregroundStyle(Color.Nym.textSecondary)
                .font(.system(size: 18, weight: .bold))
        }
    }

    @ViewBuilder
    func titleSubtitle() -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(viewModel.title)
                .foregroundStyle(Color.Nym.textPrimary)
                .textStyle(.Body.Large.regular)

            if let subtitle = viewModel.subtitle {
                Text(subtitle)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .textStyle(.Body.Small.regular)
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
        if case let .toggle(_, isDisabled) = viewModel.accessory {
            Toggle("", isOn: $isToggleOn)
                .toggleStyle(.switch)
                .tint(Color.Nym.primary)
                .labelsHidden()
                .disabled(isDisabled)
                .padding(.trailing, 16)
                .accessibilityLabel("\(viewModel.title) \(viewModel.subtitle ?? "")")
                .accessibilityHint(viewModel.accessory.accessibilityHint)
        }
    }

    @ViewBuilder
    func optionalMultilineLabel() -> some View {
        if let multilineText = viewModel.multilineText {
            HStack(spacing: 0) {
                Text(multilineText)
                    .textStyle(.Body.Medium.regular)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .padding(.horizontal, 16)
                    .tint(Color.Nym.textSecondary)
                Spacer()
            }
            Spacer()
                .frame(height: 18)
        }
    }

    @ViewBuilder
    func optionalCustomContent() -> some View {
        if let customContent {
            customContent()
        }
    }
}

private extension View {
    @ViewBuilder
    func settingsListAccessibility(_ viewModel: SettingsListItemViewModel, combineChildren: Bool) -> some View {
        if combineChildren {
            accessibilityElement(children: .combine)
                .accessibilityLabel("\(viewModel.title) \(viewModel.subtitle ?? "")")
                .accessibilityValue(viewModel.accessory.accessibilityValue)
                .accessibilityHint(viewModel.accessory.accessibilityHint)
                .accessibilityAddTraits([.isButton])
        } else {
            accessibilityElement(children: .contain)
        }
    }
}
