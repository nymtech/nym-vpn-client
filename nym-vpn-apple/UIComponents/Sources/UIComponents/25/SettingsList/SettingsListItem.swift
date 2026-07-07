import SwiftUI
import AppSettings
import Theme

public struct SettingsListItem: View {
    @ObservedObject private var viewModel: SettingsListItemViewModel

    @State private var isHovered = false
    @State private var isToggleOn = false

    public init(viewModel: SettingsListItemViewModel) {
        self.viewModel = viewModel
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
                HStack(spacing: 0) {
                    iconImage()
                        .padding(.leading, NymSpacing.large)
                    titleSubtitle()
                        .padding(.leading, NymSpacing.large)
                        .padding(.trailing, NymSpacing.small)
                    Spacer()
                }
                .frame(maxHeight: .infinity)
                .contentShape(Rectangle())
                .onTapGesture {
                    if case let .toggle(_, isDisabled) = viewModel.accessory, !isDisabled {
                        isToggleOn.toggle()
                    }
                    viewModel.action()
                }
                HStack(spacing: 0) {
                    optionalBadge()
                    optionalAccessoryImage()
                    optionalToggleView()
                }
            }
            .frame(height: 64)
            optionalMultilineLabel()
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
                .overlay(Color.Nym.divider)
        }
    }

    @ViewBuilder
    func iconImage() -> some View {
        if let imageName = viewModel.imageName {
            Image(imageName, bundle: .module)
                .resizable()
                .scaledToFit()
                .foregroundStyle(Color.Nym.textSecondary)
                .frame(width: 24, height: 24)
        } else if let systemImageName = viewModel.systemImageName {
            Image(systemName: systemImageName)
                .renderingMode(.template)
                .foregroundStyle(Color.Nym.textSecondary)
                .font(.system(size: 18, weight: .bold))
                .frame(width: 24, height: 24)
        }
    }

    @ViewBuilder
    func titleSubtitle() -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(viewModel.title)
                .foregroundStyle(Color.Nym.textPrimary)
                .nymTextStyle(.bodyDefault)

            if let subtitle = viewModel.subtitle {
                Text(subtitle)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)
            }
        }
    }

    @ViewBuilder
    func optionalBadge() -> some View {
        if let badge = viewModel.badge {
            BetaBadge(text: badge, action: viewModel.action)
                .padding(.trailing, NymSpacing.large)
        }
    }

    @ViewBuilder
    func optionalAccessoryImage() -> some View {
        if let imageName = viewModel.accessory.imageName {
            Image(imageName, bundle: .module)
                .resizable()
                .frame(width: 24, height: 24)
                .foregroundStyle(viewModel.accessory.imageColor)
                .padding(.trailing, NymSpacing.large)
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
                .padding(.trailing, NymSpacing.large)
        }
    }

    @ViewBuilder
    func optionalMultilineLabel() -> some View {
        if let multilineText = viewModel.multilineText {
            HStack(spacing: 0) {
                Text(multilineText)
                    .nymTextStyle(.bodySmall)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .padding(.horizontal, NymSpacing.large)
                    .tint(Color.Nym.textSecondary)
                Spacer()
            }
            Spacer()
                .frame(height: 18)
        }
    }
}
