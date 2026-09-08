#if os(macOS)
import SwiftUI
import ConnectionManager
import ConnectionTypes
import GRPCManager
import Theme
import UIComponents

public struct GeoExclusionView: View {
    @StateObject private var viewModel: GeoExclusionViewModel
    @FocusState private var portFieldFocused: Bool

    public init(viewModel: GeoExclusionViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    public var body: some View {
        VStack(spacing: 0) {
            CustomNavBar(
                title: "settings.geoExclusion".localizedString,
                leftButton: CustomNavBarButton(type: .back) { viewModel.navigateBack() }
            )

            ScrollView {
                VStack(spacing: 16) {
                    if viewModel.failedToStart {
                        warningBanner("geoExclusion.failedToStart".localizedString)
                    }
                    enableToggle

                    if viewModel.isEnabled {
                        warningBanner("geoExclusion.warning".localizedString)
                        socks5PortCard
                        excludedRegionsSection
                        setupInstructionsRow
                    } else {
                        descriptionCard
                        betaNote
                    }
                }
                .padding(.vertical, 24)
                .frame(maxWidth: MagicNumbers.maxWidth)
            }
            .scrollIndicators(.never)
            .padding(.horizontal, 16)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
        .task {
            await viewModel.loadState()
        }
    }
}

// MARK: - Sections -
private extension GeoExclusionView {
    var enableToggle: some View {
        let binding = Binding<Bool>(
            get: { viewModel.isEnabled },
            set: { viewModel.setEnabled($0) }
        )
        return SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(isOn: binding, isDisabled: viewModel.isLoading),
                title: "geoExclusion.enable".localizedString,
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {}
            )
        )
        .id(viewModel.isEnabled)
    }

    var descriptionCard: some View {
        HStack {
            Text("geoExclusion.description".localizedString)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            Spacer(minLength: 0)
        }
        .padding(16)
        .frame(maxWidth: .infinity)
        .background(RoundedRectangle(cornerRadius: 12).fill(Color.Nym.surface))
    }

    var betaNote: some View {
        HStack {
            Text("geoExclusion.betaNote".localizedString)
                .foregroundStyle(Color.Nym.textTertiary)
                .nymTextStyle(.bodySmall)
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 4)
    }

    var socks5PortCard: some View {
        VStack(spacing: 0) {
            Button(action: { viewModel.copyServer() }) {
                HStack(spacing: 0) {
                    Text("geoExclusion.server".localizedString)
                        .foregroundStyle(Color.Nym.textSecondary)
                        .nymTextStyle(.bodyDefault)
                    Spacer()
                    Text(viewModel.serverAddress)
                        .foregroundStyle(Color.Nym.textPrimary)
                        .font(Font.custom("Courier New", size: 15))
                        .kerning(0.3)
                        .padding(.trailing, 12)
                    GenericImage(imageName: viewModel.serverCopied ? "checkmarkSeeThrough" : "copy")
                        .frame(width: 24, height: 24)
                }
                .padding(16)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            Divider()
                .frame(height: 1)
                .overlay(Color.Nym.divider)

            Button(action: { viewModel.copyPort() }) {
                HStack(spacing: 0) {
                    Text("geoExclusion.socks5Port".localizedString)
                        .foregroundStyle(Color.Nym.textSecondary)
                        .nymTextStyle(.bodyDefault)
                    Spacer()
                    Text(String(viewModel.listenPort))
                        .foregroundStyle(Color.Nym.textPrimary)
                        .font(Font.custom("Courier New", size: 15))
                        .kerning(0.3)
                        .padding(.trailing, 12)
                    GenericImage(imageName: viewModel.portCopied ? "checkmarkSeeThrough" : "copy")
                        .frame(width: 24, height: 24)
                }
                .padding(16)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            Divider()
                .frame(height: 1)
                .overlay(Color.Nym.divider)

            VStack(alignment: .leading, spacing: 8) {
                Text("geoExclusion.customPort".localizedString)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)

                TextField("", text: $viewModel.portText)
                    .textFieldStyle(.plain)
                    .nymTextStyle(.bodyDefault)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .padding(12)
                    .background(RoundedRectangle(cornerRadius: 8).fill(Color.Nym.background))
                    .overlay(
                        RoundedRectangle(cornerRadius: 8)
                            .stroke(viewModel.portError != nil ? Color.Nym.warning : Color.Nym.border, lineWidth: 1)
                    )
                    .focused($portFieldFocused)
                    .onChange(of: viewModel.portText) { _, _ in
                        viewModel.portTextChanged()
                    }
                    .onSubmit { viewModel.commitPort() }
                    .onChange(of: portFieldFocused) { _, focused in
                        if !focused { viewModel.commitPort() }
                    }

                Text(viewModel.portError ?? "geoExclusion.portRange".localizedString)
                    .foregroundStyle(viewModel.portError != nil ? Color.Nym.warning : Color.Nym.textTertiary)
                    .nymTextStyle(.bodySmall)
            }
            .padding(16)
        }
        .frame(maxWidth: .infinity)
        .background(RoundedRectangle(cornerRadius: 12).fill(Color.Nym.surface))
    }

    var excludedRegionsSection: some View {
        VStack(spacing: 0) {
            sectionHeader("geoExclusion.excludedRegions")

            VStack(spacing: 0) {
                HStack {
                    Text(viewModel.excludedCountryName)
                        .foregroundStyle(Color.Nym.textPrimary)
                        .nymTextStyle(.bodyDefault)
                    Spacer()
                }
                .padding(16)

                Divider()
                    .frame(height: 1)
                    .overlay(Color.Nym.divider)

                HStack(spacing: 8) {
                    Image(systemName: "plus.circle.dashed")
                        .foregroundStyle(Color.Nym.textDisabled)
                        .font(.system(size: 18))
                    Text("geoExclusion.addRegion".localizedString)
                        .foregroundStyle(Color.Nym.textDisabled)
                        .nymTextStyle(.bodyDefault)
                    Spacer()
                }
                .padding(16)
            }
            .frame(maxWidth: .infinity)
            .background(RoundedRectangle(cornerRadius: 12).fill(Color.Nym.surface))
        }
    }

    var setupInstructionsRow: some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .arrow,
                title: "geoExclusion.setupInstructions".localizedString,
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: { viewModel.navigateToSetup() }
            )
        )
    }

    func sectionHeader(_ key: String) -> some View {
        HStack {
            Text(key.localizedString.uppercased())
                .foregroundStyle(Color.Nym.primary)
                .nymTextStyle(.bodySmallBold)
            Spacer()
        }
        .padding(.bottom, 8)
    }

    func warningBanner(_ text: String) -> some View {
        HStack(spacing: 0) {
            Rectangle()
                .fill(Color.Nym.warning)
                .frame(width: 3)
            HStack(alignment: .top, spacing: 8) {
                Text("⚠️")
                Text(text)
                    .foregroundStyle(Color.Nym.warning)
                    .nymTextStyle(.bodySmall)
                Spacer(minLength: 0)
            }
            .padding(12)
        }
        .frame(maxWidth: .infinity)
        .background(Color.Nym.warning.opacity(0.12))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}
#endif
