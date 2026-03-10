#if os(macOS)
import SwiftUI
import GRPCManager
import Theme
import UIComponents
import UniformTypeIdentifiers

struct DiagnosticToolView: View {
    @EnvironmentObject private var grpcManager: GRPCManager

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
            NymColor.background
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
        if reportText != nil {
            GenericButton(
                title: "settings.diagnosticTool.share".localizedString,
                style: .accentBorderOnly
            )
            .onTapGesture {
                exportReport()
            }
        }
    }

    @ViewBuilder
    func reportSection() -> some View {
        if let reportText {
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Text("settings.diagnosticTool.report".localizedString)
                        .foregroundStyle(NymColor.primary)
                        .textStyle(.Body.Large.regular)

                    Spacer()

                    CopyButton {
                        NSPasteboard.general.clearContents()
                        NSPasteboard.general.setString(reportText, forType: .string)
                    }
                }

                Text(reportText)
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Small.regular)
                    .textSelection(.enabled)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(16)
            .background(NymColor.elevation)
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
            do {
                let result = try await grpcManager.runDiagnostic()
                await MainActor.run {
                    reportText = result
                    isLoading = false
                }
            } catch {
                print("Diagnostic error: \(error)")
                await MainActor.run {
                    isLoading = false
                }
            }
        }
    }

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
}
#endif
