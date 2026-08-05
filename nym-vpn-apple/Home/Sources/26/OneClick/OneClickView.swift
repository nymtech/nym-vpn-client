import SwiftUI
import ConnectionTypes
import Theme
import UIComponents

public struct OneClickView: View {
    @Bindable var viewModel: OneClickViewModel
    private let onSelectEntry: () -> Void
    private let onSelectExit: () -> Void
    private let onShowGatewayDetails: (GatewayNode, HopType) -> Void

    @Environment(\.accessibilityVoiceOverEnabled)
    private var voiceOverEnabled
    @State private var animatedDisplayMode: OneClickDisplayMode = .powerUser

    public init(
        viewModel: OneClickViewModel,
        onSelectEntry: @escaping () -> Void = {},
        onSelectExit: @escaping () -> Void = {},
        onShowGatewayDetails: @escaping (GatewayNode, HopType) -> Void = { _, _ in }
    ) {
        self.viewModel = viewModel
        self.onSelectEntry = onSelectEntry
        self.onSelectExit = onSelectExit
        self.onShowGatewayDetails = onShowGatewayDetails
    }

    public var body: some View {
        baseSection
            .clipped()
            .onAppear { animatedDisplayMode = viewModel.displayMode }
            .onChange(of: viewModel.displayMode) { _, newMode in
                withAnimation(Constants.Animation.spring) {
                    animatedDisplayMode = newMode
                }
            }
#if os(iOS)
            .simultaneousGesture(
                DragGesture(minimumDistance: Constants.Gesture.dragThreshold)
                    .onEnded { value in
                        if value.translation.height < 0 {
                            viewModel.upCaretTapped()
                        } else {
                            viewModel.downCaretTapped()
                        }
                    }
            )
#endif
    }
}

private extension OneClickView {
    var baseSection: some View {
        VStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 0) {
                if animatedDisplayMode == .nerd {
                    exitNodeLabel
                        .transition(.move(edge: .top).combined(with: .opacity))
                }
                compactServerInfo
                if animatedDisplayMode == .nerd {
                    nerdEntrySection
                        .accessibilityHidden(viewModel.displayMode != .nerd)
                        .transition(.move(edge: .top).combined(with: .opacity))
                }
            }
            connectDivider
            connectButton
        }
        .padding(.horizontal, NymSpacing.standard)
        .padding(.top, NymSpacing.medium)
        .padding(.bottom, NymSpacing.component)
    }

    var connectDivider: some View {
        Divider()
            .background(Color.Nym.textTertiary.opacity(0.35))
            .padding(.vertical, NymSpacing.standard)
    }

    var exitNodeLabel: some View {
        Text("oneClick.exitNode.label".localizedString)
            .nymTextStyle(.bodySmall)
            .foregroundStyle(Color.Nym.textSecondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.bottom, NymSpacing.small)
            .accessibilityHidden(viewModel.displayMode != .nerd)
    }

    @ViewBuilder var compactServerInfo: some View {
        Group {
            switch viewModel.selectionPhase {
            case .selecting:
                selectingRowCompact(score: .offline)
            case let .selected(info):
                selectedRowCompact(info: info, showCarets: true)
            }
        }
        .contentShape(Rectangle())
        .onTapGesture { onSelectExit() }
        .accessibilityAddTraits(.isButton)
    }

    func selectingRowCompact(score: OneClickServerScore) -> some View {
        HStack(alignment: .center, spacing: NymSpacing.medium) {
            scoreBars(score: score)
            randomGlyph
            Text("gatewaysView.random".localizedString)
                .nymTextStyle(.bodySmall)
                .foregroundStyle(Color.Nym.textPrimary)
            Spacer()
            caretColumn
        }
    }

    var randomGlyph: some View {
        GenericImage(imageName: "random")
            .foregroundStyle(Color.Nym.textPrimary)
            .frame(width: Constants.FlagImage.size, height: Constants.FlagImage.size)
            .accessibilityHidden(true)
    }

    @ViewBuilder var caretColumn: some View {
        if animatedDisplayMode == .nerd {
            downCaretView
        } else {
            upCaretView
        }
    }

    func selectedRowCompact(info: OneClickServerInfo, showCarets: Bool) -> some View {
        let primaryText = info.title
        let secondaryText: String? = (info.subtitle?.isEmpty ?? true) ? nil : info.subtitle
        return HStack(alignment: .center, spacing: NymSpacing.medium) {
            scoreBars(score: info.score)
            flagImage(
                countryCode: info.countryCode,
                isRandomSelection: info.isRandomSelection,
                isSafestSelection: info.isSafestSelection
            )
            ZStack(alignment: .leading) {
                VStack(alignment: .leading, spacing: 0) {
                    Text("X")
                    Text("X")
                }
                .nymTextStyle(.bodySmall)
                .hidden()
                VStack(alignment: .leading, spacing: 0) {
                    Text(primaryText)
                        .nymTextStyle(.bodySmall)
                        .foregroundStyle(Color.Nym.textPrimary)
                        .lineLimit(1)
                        .truncationMode(.tail)
                    if let secondaryText {
                        Text(secondaryText)
                            .nymTextStyle(.bodySmall)
                            .foregroundStyle(Color.Nym.textSecondary)
                            .lineLimit(1)
                            .truncationMode(.tail)
                    }
                }
            }
            Spacer()
            if info.showsInfoButton, let gateway = info.gateway, let hopType = info.hopType {
                gatewayDetailsButton(gateway: gateway, hopType: hopType)
            }
            if showCarets {
                caretColumn
            }
        }
    }

    func gatewayDetailsButton(gateway: GatewayNode, hopType: HopType) -> some View {
        Button {
            onShowGatewayDetails(gateway, hopType)
        } label: {
            Image(systemName: "info.circle")
                .foregroundStyle(Color.Nym.textSecondary)
                .frame(width: Constants.InfoIcon.size, height: Constants.InfoIcon.size)
        }
        .buttonStyle(.plain)
        .contentShape(Rectangle())
        .focusEffectDisabled(!voiceOverEnabled)
#if os(macOS)
        .focusable(voiceOverEnabled)
#endif
        .accessibilityLabel(Text("oneClick.server.details.accessibilityLabel".localizedString))
    }

    @ViewBuilder
    func flagImage(countryCode: String, isRandomSelection: Bool = false, isSafestSelection: Bool = false) -> some View {
        if countryCode.isEmpty {
            if isSafestSelection {
                GenericImage(imageName: "safest")
                    .foregroundStyle(Color.Nym.textSecondary)
                    .frame(width: Constants.FlagImage.size, height: Constants.FlagImage.size)
                    .accessibilityHidden(true)
            } else if isRandomSelection {
                GenericImage(imageName: "random")
                    .foregroundStyle(Color.Nym.textSecondary)
                    .frame(width: Constants.FlagImage.size, height: Constants.FlagImage.size)
                    .accessibilityHidden(true)
            } else {
                Image(systemName: "globe")
                    .font(.system(size: Constants.FlagImage.size))
                    .foregroundStyle(Color.Nym.textSecondary)
                    .frame(width: Constants.FlagImage.size, height: Constants.FlagImage.size)
                    .accessibilityHidden(true)
            }
        } else {
            GenericImage(imageName: countryCode.lowercased())
                .frame(width: Constants.FlagImage.size, height: Constants.FlagImage.size)
                .clipShape(Circle())
                .accessibilityHidden(true)
        }
    }

    func scoreBars(score: OneClickServerScore) -> some View {
        Image(systemName: "cellularbars", variableValue: score.variableValue)
            .symbolRenderingMode(.palette)
            .foregroundStyle(score.activeColor, Color.Nym.textTertiary.opacity(0.35))
            .font(.system(size: Constants.ScoreBars.iconSize))
            .frame(width: Constants.ScoreBars.frameSize, height: Constants.ScoreBars.frameSize)
            .accessibilityHidden(true)
    }

    var upCaretView: some View {
        ImageButton(
            systemImageName: "chevron.up",
            imageSize: Constants.CaretButton.imageSize,
            accessibilityLabel: "oneClick.upCaret.accessibilityLabel".localizedString,
            renderSize: Constants.CaretButton.renderSize,
            layoutSize: Constants.CaretButton.layoutSize,
            accessibilityHint: "oneClick.upCaret.accessibilityHint".localizedString
        ) {
            viewModel.upCaretTapped()
        }
        .foregroundStyle(Color.Nym.textTertiary)
    }

    var downCaretView: some View {
        ImageButton(
            systemImageName: "chevron.down",
            imageSize: Constants.CaretButton.imageSize,
            accessibilityLabel: "oneClick.downCaret.accessibilityLabel".localizedString,
            renderSize: Constants.CaretButton.renderSize,
            layoutSize: Constants.CaretButton.layoutSize,
            accessibilityHint: "oneClick.downCaret.accessibilityHint".localizedString
        ) {
            viewModel.downCaretTapped()
        }
        .foregroundStyle(Color.Nym.textTertiary)
    }

    @ViewBuilder var nerdEntrySection: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("oneClick.entryNode.label".localizedString)
                .nymTextStyle(.bodySmall)
                .foregroundStyle(Color.Nym.textSecondary)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.bottom, animatedDisplayMode == .nerd ? NymSpacing.small : 0)
                .padding(.top, NymSpacing.medium)
            Group {
                switch viewModel.entrySelectionPhase {
                case .selecting:
                    selectingEntryRowCompact
                case let .selected(info):
                    selectedRowCompact(info: info, showCarets: false)
                }
            }
            .contentShape(Rectangle())
            .onTapGesture {
                guard viewModel.displayMode == .nerd else { return }
                onSelectEntry()
            }
            .accessibilityAddTraits(viewModel.displayMode == .nerd ? .isButton : [])
        }
    }

    var selectingEntryRowCompact: some View {
        HStack(alignment: .center, spacing: NymSpacing.medium) {
            scoreBars(score: .offline)
            randomGlyph
            Text("gatewaysView.random".localizedString)
                .nymTextStyle(.bodySmall)
                .foregroundStyle(Color.Nym.textPrimary)
            Spacer()
        }
    }

    var connectButton: some View {
        NymButton(
            connectButtonLabel,
            style: connectButtonStyle,
            cornerRadius: Constants.ConnectButton.cornerRadius,
            isDisabled: connectButtonDisabled
        ) {
            viewModel.connectButtonTapped()
        }
    }

    var connectButtonLabel: String {
        switch viewModel.connectState {
        case .disconnected:
            "oneClick.connectButton.disconnected".localizedString
        case .connecting:
            "oneClick.connectButton.connecting".localizedString
        case .stop:
            "stop".localizedString
        case .connected:
            "oneClick.connectButton.connected".localizedString
        case .disconnecting:
            "disconnecting".localizedString
        case .noInternet:
            "offline".localizedString
        case .noSubscription:
            "home.getStarted".localizedString
        }
    }

    var connectButtonStyle: NymButton.Style {
        switch viewModel.connectState {
        case .disconnected, .noSubscription:
            .primary
        case .connecting, .disconnecting, .noInternet:
            .connecting
        case .stop:
            .destructive
        case .connected:
            .connected
        }
    }

    var connectButtonDisabled: Bool {
        switch viewModel.connectState {
        case .connecting, .disconnecting, .noInternet:
            true
        case .disconnected, .stop, .connected, .noSubscription:
            false
        }
    }

    enum Constants {
        enum Animation {
            static let spring = SwiftUI.Animation.spring(response: 0.35, dampingFraction: 0.8)
        }
        enum Gesture {
            static let dragThreshold: CGFloat = 40
        }
        enum ScoreBars {
            static let iconSize: CGFloat = 20
            static let frameSize: CGFloat = 24
        }
        enum FlagImage {
            static let size: CGFloat = 20
        }
        enum InfoIcon {
            static let size: CGFloat = 20
        }
        enum CaretButton {
            static let imageSize: CGFloat = 20
            static let renderSize: CGFloat = 10
            static let layoutSize: CGFloat = 24
        }
        enum ConnectButton {
            static let cornerRadius: CGFloat = 100
        }
    }
}
