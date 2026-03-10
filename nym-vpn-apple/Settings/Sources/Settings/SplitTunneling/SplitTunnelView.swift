#if os(macOS)
import SwiftUI
import AppKit
import AppDiscoveryService
import ConnectionManager
import ConnectionTypes
import ExternalLinkManager
import ImpactGenerator
import GRPCManager
import Theme
import UIComponents

public struct SplitTunnelView: View {
    private let alphabet = Array("ABCDEFGHIJKLMNOPQRSTUVWXYZ").map(String.init) + ["#"]

    @Environment(\.appearsActive) private var appearsActive
    @Environment(AppDiscoveryService.self) private var appDiscoveryService
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var impactGenerator: ImpactGenerator
    @EnvironmentObject private var grpcManager: GRPCManager
    @EnvironmentObject private var externalLinkManager: ExternalLinkManager
    @Binding private var path: NavigationPath
    @State private var foundApps: [FoundApp]?
    @State private var isFullDiskAccessEnabled = true
    @State private var isInfoModalDisplayed = false

    private var splitTunnelConfig: SplitTunnelConfig {
        connectionManager.connectionConfig.splitTunnelConfig
    }

    public var body: some View {
        VStack(spacing: 0) {
            CustomNavBar(
                title: "settings.splitTunnel".localizedString,
                leftButton: CustomNavBarButton(type: .back) { navigateBack() },
                rightButton: CustomNavBarButton(type: .info) { isInfoModalDisplayed.toggle() }
            )

            scrollContent
                .padding(.horizontal, 16)
                .scrollIndicators(.never)
                .frame(maxWidth: MagicNumbers.maxWidth)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .overlay {
            if isInfoModalDisplayed {
                SplitTunnelInfoModal(isDisplayed: $isInfoModalDisplayed)
                    .transition(.opacity)
                    .animation(.easeInOut, value: isInfoModalDisplayed)
            }
        }
        .task {
            foundApps = appDiscoveryService.enumerateApps()

            if let needFullDiskAccess = try? await grpcManager.needFullDiskAccess() {
                isFullDiskAccessEnabled = !needFullDiskAccess
            }
        }
        .onChange(of: appearsActive) { _, isActive in
            Task {
                if isActive, let needFullDiskAccess = try? await grpcManager.needFullDiskAccess() {
                    isFullDiskAccessEnabled = !needFullDiskAccess
                }
            }
        }
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

private extension SplitTunnelView {
    var splitTunnelToggle: some View {
        SettingsListItem(
            viewModel:
                SettingsListItemViewModel(
                    accessory: .toggle(
                        isOn:
                            Binding(
                                get: { splitTunnelConfig.isEnabled },
                                set: { newValue in
                                    var config = connectionManager.connectionConfig
                                    config.splitTunnelConfig.isEnabled = newValue
                                    connectionManager.connectionConfig = config
                                }
                            ),
                        isDisabled: !isFullDiskAccessEnabled
                    ),
                    title: "settings.splitTunnel".localizedString,
                    systemImageName: "arrow.trianglehead.branch",
                    position: SettingsListItemPosition(isFirst: true, isLast: true),
                    action: {}
                )
        )
        .id(isFullDiskAccessEnabled)
    }

    var changesText: some View {
        HStack {
            Text("\("splitTunel.apps.exclude".localizedString) \n\("splitTunnel.apps.unprotected".localizedString)")
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
            Spacer()
        }
    }

    var appsText: some View {
        HStack {
            Text("splitTunnel.apps".localizedString)
                .foregroundStyle(NymColor.primary)
                .textStyle(.Headline.Small.regular)
            Spacer()
        }
    }

    @ViewBuilder var scrollContent: some View {
        if let foundApps {
            let sections = splitTunnelConfig.isEnabled ? makeSections(from: foundApps) : []

            ScrollViewReader { proxy in
                ScrollView {
                    scrollInnerContent(sections: sections)
                }
                .overlay(alignment: .bottomTrailing) {
                    if splitTunnelConfig.isEnabled {
                        SectionIndexOverlay(
                            alphabet: alphabet,
                            sections: sections,
                            scrollProxy: proxy
                        )
                    }
                }
            }
        } else {
            Spacer()
            ProgressView()
            Spacer()
        }
    }

    func scrollInnerContent(sections: [AppSection]) -> some View {
        VStack(spacing: 0) {
            Spacer()
                .frame(height: 24)
            if !isFullDiskAccessEnabled {
                fullDiskAccessSection()
                Spacer()
                    .frame(height: 24)
            }
            splitTunnelToggle
            Spacer()
                .frame(height: 24)
            if splitTunnelConfig.isEnabled {
                changesText
                Spacer()
                    .frame(height: 24)
                appsText
                Spacer()
                    .frame(height: 8)
                sectionList(sections: sections)
                    .padding(.trailing, 16)
            }
        }
    }

    func fullDiskAccessSection() -> some View {
        HStack(spacing: 0) {
            Text(fullDiskAccessAttributtedString())
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
            Spacer()
        }
    }

    func fullDiskAccessAttributtedString() -> AttributedString {
        let first = AttributedString("splitTunnel.fullDiskAccess".localizedString)
        var second = AttributedString("splitTunnel.open".localizedString)
        let third = AttributedString("splitTunnel.systemSettings".localizedString)
        let forth = AttributedString("splitTunnel.enableSystemSettings".localizedString)
        second.underlineStyle = .single
        second.foregroundColor = NymColor.accent
        second.link = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
        return first + AttributedString("\n") + second + AttributedString(" ") + third + AttributedString(" ") + forth
    }

    func sectionList(sections: [AppSection]) -> some View {
        LazyVStack(spacing: 0) {
            ForEach(sections) { section in
                VStack(spacing: 0) {
                    HStack {
                        Spacer()
                            .frame(width: 8)
                        Text(section.title)
                            .foregroundStyle(NymColor.gray1)
                            .textStyle(.Body.Medium.regular)
                        Spacer()
                    }
                    .padding(.vertical, 8)
                    .background(
                        RoundedRectangle(cornerRadius: 8)
                            .fill(NymColor.elevation)
                    )
                    .id(section.id)

                    ForEach(Array(section.apps.enumerated()), id: \.offset) { _, app in
                        appCell(for: app)
                    }
                }
                .background(NymColor.background)
            }
        }
    }

    func appCell(for app: FoundApp) -> some View {
        VStack(spacing: 0) {
            Spacer()
                .frame(height: 12)
            HStack(spacing: 0) {
                if let appBundlePath = appBundlePath(for: app) {
                    Image(nsImage: NSWorkspace.shared.icon(forFile: appBundlePath))
                        .resizable()
                        .scaledToFit()
                        .frame(width: 24, height: 24)
                } else if let iconURL = app.icon, let iconImage = NSImage(contentsOf: iconURL) {
                    Image(nsImage: iconImage)
                        .resizable()
                        .scaledToFit()
                        .frame(width: 24, height: 24)
                } else {
                    Image(systemName: "app.fill")
                        .resizable()
                        .scaledToFit()
                        .padding(6)
                        .foregroundStyle(NymColor.gray1)
                        .frame(width: 24, height: 24)
                }
                Spacer()
                    .frame(width: 16)
                Text(app.name)
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Body.Medium.regular)
                Spacer()
                appEnabledButton(isEnabled: !isAppExcluded(app)) {
                    toggleAppState(app: app)
                }
            }
            Spacer()
                .frame(height: 12)
        }
        .background(NymColor.background)
        .clipShape(Rectangle())
    }

    func appEnabledButton(isEnabled: Bool, onTap: @escaping () -> Void) -> some View {
        Button(action: onTap) {
            HStack(spacing: 0) {
                ZStack {
                    if isEnabled {
                        Color.clear
                    } else {
                        NymColor.error.opacity(0.10)
                    }
                    Image(systemName: "slash.circle.fill")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(isEnabled ? NymColor.gray1 : Color.red)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)

                Rectangle()
                    .fill(NymColor.gray2)
                    .frame(width: 1)

                ZStack {
                    if isEnabled {
                        NymColor.action.opacity(0.10)
                    } else {
                        Color.clear
                    }
                    Image(systemName: "shield.fill")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(isEnabled ? NymColor.action : NymColor.gray1)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
            .frame(width: 84, height: 24)
            .background(NymColor.elevation)
            .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .stroke(NymColor.gray2, lineWidth: 1)
            )
        }
        .buttonStyle(.plain)
    }
}

private struct AppSection: Identifiable {
    let title: String
    let apps: [FoundApp]

    var id: String { "section-\(title)" }
}

private extension SplitTunnelView {
    func makeSections(from apps: [FoundApp]) -> [AppSection] {
        let groupedApps = Dictionary(grouping: apps) { app -> String in
            guard let first = app.name.first else { return "#" }
            let uppercased = String(first).uppercased()
            return uppercased.range(of: "^[A-Z]$", options: .regularExpression) != nil ? uppercased : "#"
        }

        let sortedKeys = groupedApps.keys.sorted { lhs, rhs in
            switch (lhs, rhs) {
            case ("#", "#"):
                return false
            case ("#", _):
                return false
            case (_, "#"):
                return true
            default:
                return lhs < rhs
            }
        }

        return sortedKeys.map { key in
            let sortedApps = (groupedApps[key] ?? []).sorted {
                $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending
            }
            return AppSection(title: key, apps: sortedApps)
        }
    }

    func isAppExcluded(_ app: FoundApp) -> Bool {
        guard let path = app.executablePath else { return false }
        return splitTunnelConfig.appPaths.contains(path)
    }

    func toggleAppState(app: FoundApp) {
        guard let path = app.executablePath else { return }
        var config = connectionManager.connectionConfig
        if let index = config.splitTunnelConfig.appPaths.firstIndex(of: path) {
            config.splitTunnelConfig.appPaths.remove(at: index)
        } else {
            config.splitTunnelConfig.appPaths.append(path)
        }
        connectionManager.connectionConfig = config
    }

    func appBundlePath(for app: FoundApp) -> String? {
        guard let executablePath = app.executablePath else { return nil }
        let executableURL = URL(filePath: executablePath)

        // MyApp.app/Contents/MacOS/MyApp -> MyApp.app
        guard executableURL.pathComponents.count >= 4 else { return nil }
        let bundleURL = executableURL
            .deletingLastPathComponent() // MacOS
            .deletingLastPathComponent() // Contents
            .deletingLastPathComponent() // MyApp.app

        guard bundleURL.pathExtension == "app" else { return nil }
        return bundleURL.path
    }

    func navigateBack() {
        guard !path.isEmpty else { return }
        impactGenerator.softImpact()
        path.removeLast()
    }
}

private struct SectionIndexOverlay: View {
    let alphabet: [String]
    let sections: [AppSection]
    let scrollProxy: ScrollViewProxy

    private let letterHeight: CGFloat = 14
    private let letterSpacing: CGFloat = 8

    @State private var draggedLetter: String?
    @State private var isDragging = false

    var body: some View {
        VStack(spacing: letterSpacing) {
            ForEach(alphabet, id: \.self) { letter in
                Text(letter)
                    .textStyle(.Body.Small.bold)
                    .foregroundStyle(letterColor(for: letter))
                    .frame(width: 16, height: letterHeight)
            }
        }
        .padding(.vertical, 4)
        .padding(.horizontal, 2)
        .contentShape(Rectangle())
        .gesture(
            DragGesture(minimumDistance: 0)
                .onChanged { value in
                    isDragging = true
                    let letter = letterAtLocation(value.location)
                    if letter != draggedLetter {
                        draggedLetter = letter
                        scrollToLetter(letter)
                    }
                }
                .onEnded { _ in
                    isDragging = false
                    draggedLetter = nil
                }
        )
        .padding(.trailing, -9)
    }

    private func letterColor(for letter: String) -> Color {
        if letter == draggedLetter {
            return NymColor.primary
        }
        return sections.contains(where: { $0.title == letter })
            ? NymColor.gray1
            : NymColor.gray2
    }

    private func letterAtLocation(_ location: CGPoint) -> String {
        let verticalPadding: CGFloat = 4
        let index = Int((location.y - verticalPadding) / (letterHeight + letterSpacing))
        let clampedIndex = max(0, min(index, alphabet.count - 1))
        return alphabet[clampedIndex]
    }

    private func scrollToLetter(_ letter: String) {
        guard sections.contains(where: { $0.title == letter }) else { return }
        withAnimation(.easeInOut(duration: 0.2)) {
            scrollProxy.scrollTo("section-\(letter)", anchor: .top)
        }
    }
}
#endif
