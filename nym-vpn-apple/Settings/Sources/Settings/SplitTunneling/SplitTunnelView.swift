#if os(macOS)
import SwiftUI
import AppKit
import UniformTypeIdentifiers
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
    @State private var swipedAppPath: String?
    @State private var isImporterPresented = false
    @State private var pendingScrollID: String?

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
            Color.Nym.background
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
                                    var next = splitTunnelConfig
                                    next.isEnabled = newValue
                                    connectionManager.setSplitTunnelConfig(next)
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
            Text("⚠️ \("splitTunnel.betaFeature".localizedString)\n\("splitTunel.apps.exclude".localizedString) \n\("splitTunnel.apps.unprotected".localizedString)")
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            Spacer()
        }
    }

    var appsText: some View {
        HStack {
            Text("splitTunnel.apps".localizedString)
                .foregroundStyle(Color.Nym.textPrimary)
                .nymTextStyle(.bodyLarge)
            Spacer()
        }
    }

    var addApplicationButton: some View {
        Button {
            isImporterPresented = true
        } label: {
            HStack(spacing: 8) {
                Image(systemName: "plus")
                    .font(.system(size: 14, weight: .semibold))
                    .accessibilityHidden(true)
                Text("splitTunnel.addApplication".localizedString)
                    .nymTextStyle(.bodyDefaultBold)
            }
            .foregroundStyle(Color.Nym.primary)
            .frame(maxWidth: .infinity)
            .frame(height: 56)
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(
                        Color.Nym.primary,
                        style: StrokeStyle(lineWidth: 1, dash: [6, 4])
                    )
            )
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .fileImporter(
            isPresented: $isImporterPresented,
            allowedContentTypes: [.application],
            allowsMultipleSelection: false
        ) { result in
            if case let .success(urls) = result, let url = urls.first {
                addCustomApp(at: url)
            }
        }
        .fileDialogDefaultDirectory(URL(filePath: "/Applications"))
        .fileDialogConfirmationLabel("splitTunnel.addApplication.prompt".localizedString)
    }

    @ViewBuilder var scrollContent: some View {
        if let foundApps {
            let sections = splitTunnelConfig.isEnabled ? makeSections(from: displayApps(discovered: foundApps)) : []

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
                .onChange(of: pendingScrollID) { _, target in
                    guard let target else { return }
                    DispatchQueue.main.async {
                        withAnimation(.easeInOut(duration: 0.2)) {
                            proxy.scrollTo(target, anchor: .top)
                        }
                        pendingScrollID = nil
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
                addApplicationButton
                    .padding(.trailing, 16)
                Spacer()
                    .frame(height: 16)
                sectionList(sections: sections)
                    .padding(.trailing, 16)
            }
        }
    }

    func fullDiskAccessSection() -> some View {
        HStack(spacing: 0) {
            Text(fullDiskAccessAttributtedString())
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            Spacer()
        }
    }

    func fullDiskAccessAttributtedString() -> AttributedString {
        let first = AttributedString("splitTunnel.fullDiskAccess".localizedString)
        var second = AttributedString("splitTunnel.open".localizedString)
        let third = AttributedString("splitTunnel.systemSettings".localizedString)
        let forth = AttributedString("splitTunnel.enableSystemSettings".localizedString)
        second.underlineStyle = .single
        second.foregroundColor = Color.Nym.primary
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
                            .foregroundStyle(Color.Nym.textSecondary)
                            .nymTextStyle(.bodyDefault)
                        Spacer()
                    }
                    .padding(.vertical, 8)
                    .background(
                        RoundedRectangle(cornerRadius: 12)
                            .fill(Color.Nym.surface)
                    )
                    .id(section.id)

                    ForEach(Array(section.apps.enumerated()), id: \.offset) { _, app in
                        appCell(for: app)
                            .id(app.executablePath ?? app.name)
                    }
                }
                .background(Color.Nym.background)
            }
        }
    }

    func appCell(for app: FoundApp) -> some View {
        let path = app.executablePath
        let isSwiped = path != nil && swipedAppPath == path
        let canRemove = isCustomApp(app)

        return ZStack(alignment: .trailing) {
            if canRemove {
                Button {
                    withAnimation { swipedAppPath = nil }
                    removeCustomApp(app)
                } label: {
                    Text("splitTunnel.removeApplication".localizedString)
                        .nymTextStyle(.bodyDefaultBold)
                        .foregroundStyle(Color.Nym.error)
                        .frame(width: 80)
                        .frame(maxHeight: .infinity)
                }
                .buttonStyle(.plain)
            }

            cellContent(for: app)
                .background(Color.Nym.background)
                .offset(x: isSwiped ? -80 : 0)
                .gesture(canRemove ? swipeGesture(path: path) : nil)
                .onTapGesture {
                    if isSwiped { withAnimation { swipedAppPath = nil } }
                }
        }
        .clipShape(Rectangle())
    }

    @ViewBuilder
    func cellContent(for app: FoundApp) -> some View {
        let row = VStack(spacing: 0) {
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
                        .foregroundStyle(Color.Nym.textSecondary)
                        .frame(width: 24, height: 24)
                }
                Spacer()
                    .frame(width: 16)
                Text(app.name)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .nymTextStyle(.bodyDefault)
                Spacer()
                appEnabledButton(isEnabled: !isAppExcluded(app)) {
                    toggleAppState(app: app)
                }
            }
            Spacer()
                .frame(height: 12)
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(app.name)
        .accessibilityValue(accessibilityValue(for: app))
        .accessibilityHint("splitTunnel.accessibility.toggleHint".localizedString)
        .accessibilityAddTraits(.isButton)
        .accessibilityAction { toggleAppState(app: app) }

        if isCustomApp(app) {
            row.accessibilityAction(named: Text("splitTunnel.removeApplication".localizedString)) {
                removeCustomApp(app)
            }
        } else {
            row
        }
    }

    func accessibilityValue(for app: FoundApp) -> String {
        isAppExcluded(app)
            ? "splitTunnel.accessibility.excluded".localizedString
            : "splitTunnel.accessibility.protected".localizedString
    }

    func swipeGesture(path: String?) -> some Gesture {
        DragGesture(minimumDistance: 10)
            .onEnded { value in
                guard let path else { return }
                withAnimation {
                    if value.translation.width < -30 {
                        swipedAppPath = path
                    } else if value.translation.width > 30 {
                        swipedAppPath = nil
                    }
                }
            }
    }

    func appEnabledButton(isEnabled: Bool, onTap: @escaping () -> Void) -> some View {
        Button(action: onTap) {
            HStack(spacing: 0) {
                ZStack {
                    if isEnabled {
                        Color.clear
                    } else {
                        Color.Nym.error.opacity(0.10)
                    }
                    Image(systemName: "slash.circle.fill")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(isEnabled ? Color.Nym.textSecondary : Color.Nym.error)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)

                Rectangle()
                    .fill(Color.Nym.textSecondary)
                    .frame(width: 1)

                ZStack {
                    if isEnabled {
                        Color.Nym.primary.opacity(0.10)
                    } else {
                        Color.clear
                    }
                    Image(systemName: "shield.fill")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(isEnabled ? Color.Nym.primary : Color.Nym.textSecondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
            .frame(width: 84, height: 24)
            .background(Color.Nym.surface)
            .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .stroke(Color.Nym.textSecondary, lineWidth: 1)
            )
            // Tooltip lives on the filled label content (solid hover region);
            // on the outer plain Button it didn't register. State-dependent:
            // protected → "Exclude", excluded → "Include".
            .contentShape(Rectangle())
            .help(
                isEnabled
                    ? "splitTunnel.exclude.tooltip".localizedString
                    : "splitTunnel.include.tooltip".localizedString
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
    func sectionKey(for app: FoundApp) -> String {
        guard let first = app.name.first else { return "#" }
        let uppercased = String(first).uppercased()
        return uppercased.range(of: "^[A-Z]$", options: .regularExpression) != nil ? uppercased : "#"
    }

    func makeSections(from apps: [FoundApp]) -> [AppSection] {
        let groupedApps = Dictionary(grouping: apps) { sectionKey(for: $0) }

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
        var next = splitTunnelConfig
        if next.appPaths.contains(path) {
            next.appPaths.remove(path)
        } else {
            next.appPaths.insert(path)
        }
        connectionManager.setSplitTunnelConfig(next)
    }

    func isCustomApp(_ app: FoundApp) -> Bool {
        guard let path = app.executablePath else { return false }
        return splitTunnelConfig.customAppPaths.contains(path)
    }

    func removeCustomApp(_ app: FoundApp) {
        guard let path = app.executablePath else { return }
        var next = splitTunnelConfig
        next.customAppPaths.remove(path)
        next.appPaths.remove(path)
        connectionManager.setSplitTunnelConfig(next)
    }

    func addCustomApp(at url: URL) {
        guard url.lastPathComponent != "NymVPN.app" else { return }

        let didAccess = url.startAccessingSecurityScopedResource()
        defer { if didAccess { url.stopAccessingSecurityScopedResource() } }

        let app = appDiscoveryService.foundApp(at: url)
        guard let path = app.executablePath else { return }

        var next = splitTunnelConfig
        next.customAppPaths.insert(path)
        next.appPaths.insert(path)
        connectionManager.setSplitTunnelConfig(next)

        // Scroll to the new app's section
        pendingScrollID = "section-\(sectionKey(for: app))"
    }

    func bundleURL(fromExecutablePath executablePath: String) -> URL? {
        let executableURL = URL(filePath: executablePath)
        guard executableURL.pathComponents.count >= 4 else { return nil }
        let bundleURL = executableURL
            .deletingLastPathComponent() // MacOS
            .deletingLastPathComponent() // Contents
            .deletingLastPathComponent() // Foo.app
        return bundleURL.pathExtension == "app" ? bundleURL : nil
    }

    func appBundlePath(for app: FoundApp) -> String? {
        guard let executablePath = app.executablePath else { return nil }
        return bundleURL(fromExecutablePath: executablePath)?.path
    }

    func bundleURL(forExecutable executablePath: String) -> URL {
        bundleURL(fromExecutablePath: executablePath) ?? URL(filePath: executablePath)
    }

    func displayApps(discovered: [FoundApp]) -> [FoundApp] {
        var byPath: [String: FoundApp] = [:]
        for app in discovered {
            if let path = app.executablePath { byPath[path] = app }
        }
        for path in splitTunnelConfig.customAppPaths where byPath[path] == nil {
            let resolved = appDiscoveryService.foundApp(at: bundleURL(forExecutable: path))
            if resolved.executablePath != nil {
                byPath[path] = resolved
            } else {
                // Bundle no longer resolvable (e.g. deleted) — keep a removable placeholder
                // that still carries the config path so isCustomApp/removeCustomApp work.
                let name = bundleURL(forExecutable: path).deletingPathExtension().lastPathComponent
                byPath[path] = FoundApp(name: name, executablePath: path, icon: resolved.icon)
            }
        }
        return Array(byPath.values)
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
                    .nymTextStyle(.bodySmallBold)
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
        .accessibilityHidden(true)
    }

    private func letterColor(for letter: String) -> Color {
        guard sections.contains(where: { $0.title == letter })
        else {
            return Color.Nym.textDisabled
        }
        return letter == draggedLetter ? Color.Nym.textPrimary : Color.Nym.textSecondary
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
