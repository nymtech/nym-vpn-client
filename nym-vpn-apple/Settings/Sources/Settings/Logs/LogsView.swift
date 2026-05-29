import SwiftUI
import AppSettings
import Constants
import Theme
import UIComponents

public struct LogsView: View {
    @StateObject private var viewModel: LogsViewModel
    @State var isExportButtonHovered = false
    @State var isDeleteButtonHovered = false

    public init(viewModel: @autoclosure @escaping () -> LogsViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel())
    }

    public var body: some View {
        VStack(spacing: .zero) {
            navbar()

            VStack(spacing: .zero) {
                if !viewModel.logLines.isEmpty {
                    logsView()
                } else {
                    noLogsView()
                }
                logTypePicker()
            }
            .frame(maxWidth: .infinity)
            .background {
                Color.Nym.background
            }
            buttonsSection()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color.Nym.background)
        .overlay {
            if viewModel.isDeleteDialogDisplayed {
                LogsDeleteConfirmationDialog(
                    viewModel: LogsDeleteConfirmationDialogViewModel(
                        isDisplayed: $viewModel.isDeleteDialogDisplayed,
                        impactGenerator: .shared,
                        action: {
                            viewModel.deleteLogs()
                            viewModel.isDeleteDialogDisplayed = false
                        }
                    )
                )
            }
        }
        .modifier(LogsExportModifier(viewModel: viewModel))
    }
}

private extension LogsView {
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
        .padding(0)
    }

    func button(systemImageName: String, title: String) -> some View {
        VStack {
            Image(systemName: systemImageName)
                .foregroundStyle(Color.Nym.textPrimary)
                .frame(width: 24, height: 24)
                .padding(8)

            Text(title)
                .foregroundStyle(Color.Nym.textPrimary)
                .nymTextStyle(.bodyLarge)
        }
        .contentShape(RoundedRectangle(cornerRadius: 12))
        .padding(EdgeInsets(top: 4, leading: 16, bottom: 4, trailing: 16))
    }

    func exportButton() -> some View {
        button(systemImageName: "square.and.arrow.up", title: viewModel.exportLocalizedString)
            .background(
                isExportButtonHovered ? Color.Nym.surfaceAlt : Color.Nym.surface,
                in: RoundedRectangle(cornerRadius: 12)
            )
            .opacity(viewModel.isPreparingExport ? 0.5 : 1.0)
            .disabled(viewModel.logLines.isEmpty || viewModel.isPreparingExport)
            .onTapGesture {
#if os(iOS)
                viewModel.impactGenerator.impact()
#endif
                guard !viewModel.logLines.isEmpty, !viewModel.isPreparingExport else { return }
                viewModel.prepareExport()
            }
    }

    func deleteButton() -> some View {
        button(systemImageName: "trash", title: viewModel.deleteLocalizedString)
            .disabled(viewModel.logLines.isEmpty)
            .onTapGesture {
#if os(iOS)
                viewModel.impactGenerator.impact()
#endif
                if !viewModel.logLines.isEmpty {
                    viewModel.isDeleteDialogDisplayed.toggle()
                }
            }
            .background(
                isDeleteButtonHovered ? Color.Nym.surfaceAlt : Color.Nym.surface,
                in: RoundedRectangle(cornerRadius: 12)
            )
    }

    func buttonsSection() -> some View {
        HStack {
            Spacer()
            exportButton()
                .onHover { newValue in
                    isExportButtonHovered = newValue
                }
            Spacer()
            deleteButton()
                .onHover { newValue in
                    isDeleteButtonHovered = newValue
                }
            Spacer()
        }
        .background {
            Color.Nym.surface
        }
        .frame(minHeight: 80)
    }

    @ViewBuilder
    func logTypePicker() -> some View {
        if viewModel.logFileTypes.count > 1 {
            Picker("", selection: $viewModel.currentLogFileType) {
                ForEach(viewModel.logFileTypes, id: \.self) {
                    Text($0.rawValue.capitalized.localizedString)
                }
            }
            .pickerStyle(.segmented)
            .padding(16)
            .frame(maxWidth: MagicNumbers.maxWidth)
        }
    }

    func noLogsView() -> some View {
        VStack {
            Spacer()
            Text(viewModel.noLogsLocalizedString)
            Spacer()
        }
    }

    @ViewBuilder
    func logsView() -> some View {
        VStack(spacing: 0) {
            if viewModel.hasReachedLimit {
                Button {
                    viewModel.loadOlder()
                } label: {
                    Text("Load older entries")
                        .nymTextStyle(.bodyDefault)
                        .foregroundStyle(Color.Nym.info)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 8)
                }
                .buttonStyle(.plain)
                .background(Color.Nym.surface)
            }

            LogTextView(
                text: viewModel.logText,
                scrollIntent: viewModel.scrollIntent
            )
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }
}

private struct LogsExportModifier: ViewModifier {
    @ObservedObject var viewModel: LogsViewModel

    func body(content: Content) -> some View {
#if os(iOS)
        content.sheet(isPresented: $viewModel.isShareSheetPresented) {
            if let url = viewModel.exportZipURL {
                LogShareSheet(items: [url])
            }
        }
#elseif os(macOS)
        content.fileExporter(
            isPresented: $viewModel.isFileExporterPresented,
            document: viewModel.exportZipURL.map { ZipFile(url: $0) },
            contentType: .zip,
            defaultFilename: "nym-vpn-logs.zip"
        ) { _ in }
#else
        content
#endif
    }
}
