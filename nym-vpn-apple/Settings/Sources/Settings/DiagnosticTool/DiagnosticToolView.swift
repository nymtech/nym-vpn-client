import SwiftUI
import Theme
import UIComponents
#if os(macOS)
import GRPCManager
import UniformTypeIdentifiers
#elseif os(iOS)
import UIKit
import ConnectionManager
#endif

struct DiagnosticToolView: View {
#if os(macOS)
    @EnvironmentObject private var grpcManager: GRPCManager
#endif

    @Binding private var path: NavigationPath
    @State private var reportText: String?
    @State private var isLoading = false
    init(path: Binding<NavigationPath>) {
        _path = path
    }

    var body: some View {
        VStack(spacing: 0) {
            CustomNavBar(
                title: "settings.diagnosticTool".localizedString,
                leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
            )
            ScrollView {
                VStack(spacing: 24) {
                    runButton()
                    shareButton()
                    reportSection()
                }
                .padding(.horizontal, 16)
                .padding(.top, 24)
            }
            .scrollIndicators(.never)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
    }
}

private extension DiagnosticToolView {
    func runButton() -> some View {
        GenericButton(
            title: "settings.diagnosticTool.run".localizedString,
            isLoading: $isLoading
        )
        .onTapGesture {
            guard !isLoading else { return }
            runDiagnostics()
        }
    }

    @ViewBuilder
    func shareButton() -> some View {
        if let reportText {
#if os(macOS)
            GenericButton(
                title: "settings.diagnosticTool.share".localizedString,
                style: .accentBorderOnly
            )
            .onTapGesture {
                exportReport()
            }
#elseif os(iOS)
            ShareLink(item: reportText) {
                GenericButton(
                    title: "settings.diagnosticTool.share".localizedString,
                    style: .accentBorderOnly
                )
            }
            .buttonStyle(.plain)
#endif
        }
    }

    @ViewBuilder
    func reportSection() -> some View {
        if let reportText {
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Text("settings.diagnosticTool.report".localizedString)
                        .foregroundStyle(Color.Nym.textPrimary)
                        .nymTextStyle(.bodyLarge)

                    Spacer()

                    CopyButton {
                        copyReport(reportText)
                    }
                }

                Text(reportText)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)
                    .textSelection(.enabled)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(16)
            .background(Color.Nym.surface)
            .cornerRadius(8)
        }
    }
}

private extension DiagnosticToolView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func runDiagnostics() {
        isLoading = true
        Task {
            let result: String?
#if os(macOS)
            result = try? await grpcManager.runDiagnostic()
#elseif os(iOS)
            result = await ConnectionManager.shared.runDiagnostic()
#endif
            await MainActor.run {
                reportText = formatJSON(result)
                    ?? "settings.diagnosticTool.failed".localizedString
                isLoading = false
            }
        }
    }

    func formatJSON(_ jsonString: String?) -> String? {
        guard
            let jsonString,
            let data = jsonString.data(using: .utf8),
            let jsonObject = try? JSONSerialization.jsonObject(with: data),
            let prettyData = try? JSONSerialization.data(withJSONObject: jsonObject, options: [.prettyPrinted, .sortedKeys]),
            let prettyString = String(data: prettyData, encoding: .utf8)
        else {
            return jsonString
        }
        return prettyString
    }

    func copyReport(_ reportText: String) {
#if os(macOS)
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(reportText, forType: .string)
#elseif os(iOS)
        UIPasteboard.general.string = reportText
#endif
    }

#if os(macOS)
    func exportReport() {
        guard let reportText else { return }
        let savePanel = NSSavePanel()
        savePanel.allowedContentTypes = [.json]
        savePanel.nameFieldStringValue = "diagnostic_report.json"
        savePanel.begin { response in
            guard response == .OK, let url = savePanel.url else { return }
            try? reportText.write(to: url, atomically: true, encoding: .utf8)
        }
    }
#endif
}
