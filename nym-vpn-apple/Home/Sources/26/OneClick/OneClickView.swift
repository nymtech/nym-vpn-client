import SwiftUI
import Theme
import UIComponents

public struct OneClickView: View {
    @Bindable var viewModel: OneClickViewModel
    private let onSelectEntry: () -> Void
    private let onSelectExit: () -> Void

    @Environment(\.colorScheme)
    private var colorScheme

    @State private var nerdSectionHeight: CGFloat = 0
    @State private var animatedDisplayMode: OneClickDisplayMode = .oneClick

    public init(
        viewModel: OneClickViewModel,
        onSelectEntry: @escaping () -> Void = {},
        onSelectExit: @escaping () -> Void = {}
    ) {
        self.viewModel = viewModel
        self.onSelectEntry = onSelectEntry
        self.onSelectExit = onSelectExit
    }

    public var body: some View {
        VStack(spacing: 0) {
            if animatedDisplayMode != .oneClick {
                powerUserSection
                    .transition(
                        .move(edge: .top).combined(with: .opacity)
                    )
            }
            baseSection
        }
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
                    guard viewModel.canChangeDisplayMode else { return }
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
    var powerUserSection: some View {
        VStack(spacing: NymSpacing.medium) {
            speedModeToggle
            NymDivider()
        }
        .padding(.horizontal, NymSpacing.standard)
        .padding(.top, NymSpacing.component)
        .padding(.bottom, NymSpacing.medium)
        .accessibilityHidden(viewModel.displayMode == .oneClick)
    }

    var baseSection: some View {
        VStack(spacing: NymSpacing.medium) {
            VStack(alignment: .leading, spacing: 0) {
                exitNodeLabel
                compactServerInfo
                nerdEntrySection
                    .fixedSize(horizontal: false, vertical: true)
                    .background(
                        GeometryReader { geo in
                            Color.clear
                                .onAppear { nerdSectionHeight = geo.size.height }
                                .onChange(of: geo.size.height) { _, newHeight in nerdSectionHeight = newHeight }
                        }
                    )
                    .frame(height: animatedDisplayMode == .nerd ? nerdSectionHeight : 0, alignment: .top)
                    .clipped()
                    .accessibilityHidden(viewModel.displayMode != .nerd)
            }
            connectButton
        }
        .padding(.horizontal, NymSpacing.standard)
        .padding(.top, NymSpacing.medium)
        .padding(.bottom, NymSpacing.component)
    }

    var exitNodeLabel: some View {
        Text("oneClick.exitNode.label".localizedString)
            .nymTextStyle(.bodySmall)
            .foregroundStyle(Color.Nym.textSecondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .frame(height: animatedDisplayMode == .nerd ? Constants.ExitLabel.visibleHeight : 0)
            .padding(.bottom, animatedDisplayMode == .nerd ? NymSpacing.small : 0)
            .opacity(animatedDisplayMode == .nerd ? 1 : 0)
            .clipped()
            .accessibilityHidden(viewModel.displayMode != .nerd)
    }

    @ViewBuilder var compactServerInfo: some View {
        Group {
            switch viewModel.selectionPhase {
            case .selecting:
                selectingRowCompact(score: .offline)
            case let .selected(info):
                if viewModel.displayMode == .oneClick && !viewModel.isLiveConnection {
                    selectingRowCompact(score: info.score, supportsPostQuantum: info.supportsPostQuantum)
                } else {
                    selectedRowCompact(info: info, showCarets: true)
                }
            }
        }
        .contentShape(Rectangle())
        .onTapGesture {
            guard viewModel.displayMode != .oneClick else { return }
            onSelectExit()
        }
        .accessibilityAddTraits(viewModel.displayMode != .oneClick ? .isButton : [])
    }

    func selectingRowCompact(score: OneClickServerScore, supportsPostQuantum: Bool = false) -> some View {
        VStack(alignment: .leading, spacing: NymSpacing.extraExtraSmall) {
            HStack(alignment: .center, spacing: NymSpacing.medium) {
                scoreBars(score: score)
                Text("oneClick.server.bestForLocation".localizedString)
                    .nymTextStyle(.bodySmall)
                    .foregroundStyle(Color.Nym.textPrimary)
                Spacer()
                if supportsPostQuantum {
                    Image(systemName: "atom")
                        .foregroundStyle(colorScheme == .dark ? Color.Nym.primary : Color.Nym.textPrimary)
                        .frame(width: 16, height: 16)
                        .accessibilityLabel(
                            Text("oneClick.server.postQuantum.accessibilityLabel".localizedString)
                        )
                }
                upCaretView
            }
            HStack(alignment: .center, spacing: NymSpacing.medium) {
                scoreBars(score: score)
                    .hidden()
                    .accessibilityHidden(true)
                HStack(spacing: NymSpacing.extraSmall) {
                    Image(systemName: "globe")
                        .font(.system(size: Constants.GlobeIcon.fontSize))
                        .foregroundStyle(Color.Nym.textSecondary)
                        .accessibilityHidden(true)
                    Text("oneClick.server.searching".localizedString)
                        .nymTextStyle(.bodySmall)
                        .foregroundStyle(Color.Nym.textSecondary)
                }
                Spacer()
                downCaretView
                    .opacity(animatedDisplayMode != .oneClick ? 1 : 0)
                    .accessibilityHidden(viewModel.displayMode == .oneClick)
            }
        }
    }

    func selectedRowCompact(info: OneClickServerInfo, showCarets: Bool, showPostQuantum: Bool = true) -> some View {
        let primaryText = info.title
        let secondaryText: String? = (info.subtitle?.isEmpty ?? true) ? nil : info.subtitle
        return VStack(alignment: .leading, spacing: 0) {
            HStack(alignment: .center, spacing: NymSpacing.medium) {
                scoreBars(score: info.score)
                flagImage(countryCode: info.countryCode)
                Text(primaryText)
                    .nymTextStyle(.bodySmall)
                    .foregroundStyle(Color.Nym.textPrimary)
                Spacer()
                if showPostQuantum && info.supportsPostQuantum {
                    Image(systemName: "atom")
                        .foregroundStyle(colorScheme == .dark ? Color.Nym.primary : Color.Nym.textPrimary)
                        .frame(width: 16, height: 16)
                        .accessibilityLabel(
                            Text("oneClick.server.postQuantum.accessibilityLabel".localizedString)
                        )
                }
                if showCarets { upCaretView }
            }
            if let secondaryText {
                HStack(alignment: .center, spacing: NymSpacing.medium) {
                    scoreBars(score: info.score)
                        .hidden()
                        .accessibilityHidden(true)
                    Color.clear
                        .frame(width: Constants.FlagImage.size, height: Constants.FlagImage.size)
                    Text(secondaryText)
                        .nymTextStyle(.bodySmall)
                        .foregroundStyle(Color.Nym.textSecondary)
                    Spacer()
                    if showCarets {
                        downCaretView
                            .opacity(animatedDisplayMode != .oneClick ? 1 : 0)
                            .accessibilityHidden(viewModel.displayMode == .oneClick)
                    }
                }
            } else if showCarets {
                HStack(alignment: .center, spacing: NymSpacing.medium) {
                    scoreBars(score: info.score)
                        .hidden()
                        .accessibilityHidden(true)
                    Color.clear
                        .frame(width: Constants.FlagImage.size, height: Constants.FlagImage.size)
                    Spacer()
                    downCaretView
                        .opacity(animatedDisplayMode != .oneClick ? 1 : 0)
                        .accessibilityHidden(viewModel.displayMode == .oneClick)
                }
            }
        }
    }

    @ViewBuilder
    func flagImage(countryCode: String) -> some View {
        if countryCode.isEmpty {
            Image(systemName: "globe")
                .font(.system(size: Constants.FlagImage.size))
                .foregroundStyle(Color.Nym.textSecondary)
                .frame(width: Constants.FlagImage.size, height: Constants.FlagImage.size)
                .accessibilityHidden(true)
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
            .foregroundStyle(score.activeColor, Color.Nym.icon.opacity(0.35))
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
        .foregroundStyle(Color.Nym.gray1)
        .opacity(caretOpacity(showWhen: animatedDisplayMode != .nerd))
        .disabled(!viewModel.canChangeDisplayMode)
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
        .foregroundStyle(Color.Nym.gray1)
        .opacity(caretOpacity(showWhen: true))
        .disabled(!viewModel.canChangeDisplayMode)
    }

    func caretOpacity(showWhen visible: Bool) -> Double {
        guard visible else { return 0 }
        return viewModel.canChangeDisplayMode ? 1 : 0.35
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
                    selectedRowCompact(info: info, showCarets: false, showPostQuantum: false)
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
        VStack(alignment: .leading, spacing: NymSpacing.extraExtraSmall) {
            HStack(alignment: .center, spacing: NymSpacing.medium) {
                scoreBars(score: .offline)
                Text("oneClick.server.bestForLocation".localizedString)
                    .nymTextStyle(.bodySmall)
                    .foregroundStyle(Color.Nym.textPrimary)
                Spacer()
            }
            HStack(alignment: .center, spacing: NymSpacing.medium) {
                scoreBars(score: .offline)
                    .hidden()
                    .accessibilityHidden(true)
                HStack(spacing: NymSpacing.extraSmall) {
                    Image(systemName: "globe")
                        .font(.system(size: Constants.GlobeIcon.fontSize))
                        .foregroundStyle(Color.Nym.textSecondary)
                        .accessibilityHidden(true)
                    Text("oneClick.server.searching".localizedString)
                        .nymTextStyle(.bodySmall)
                        .foregroundStyle(Color.Nym.textSecondary)
                }
                Spacer()
            }
        }
    }

    var speedModeToggle: some View {
        SpeedModeToggle(isFast: viewModel.isTwoHop) { newValue in
            viewModel.setTwoHop(newValue)
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
        case .noInternet:
            "noInternet".localizedString
        case .noSubscription:
            "home.getStarted".localizedString
        }
    }

    var connectButtonStyle: NymButton.Style {
        switch viewModel.connectState {
        case .disconnected, .noSubscription:
            .primary
        case .connecting, .noInternet:
            .connecting
        case .stop:
            .destructive
        case .connected:
            .connected
        }
    }

    var connectButtonDisabled: Bool {
        switch viewModel.connectState {
        case .connecting, .noInternet:
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
        enum ExitLabel {
            static let visibleHeight: CGFloat = 20
        }
        enum ScoreBars {
            static let iconSize: CGFloat = 20
            static let frameSize: CGFloat = 24
        }
        enum FlagImage {
            static let size: CGFloat = 20
        }
        enum GlobeIcon {
            static let fontSize: CGFloat = 8
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
