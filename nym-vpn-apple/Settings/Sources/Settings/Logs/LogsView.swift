import SwiftUI
import AppSettings
import Constants
import Theme
import UIComponents

public struct LogsView: View {
    @ObservedObject private var viewModel: LogsViewModel

    public init(viewModel: LogsViewModel) {
        self.viewModel = viewModel
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
                NymColor.background
            }
            buttonsSection()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.navigationBarBackground
                .ignoresSafeArea()
        }
        .overlay {
            if viewModel.isDeleteDialogDisplayed {
                LogsDeleteConfirmationDialog(
                    viewModel: LogsDeleteConfirmationDialogViewModel(
                        isDisplayed: $viewModel.isDeleteDialogDisplayed,
                        action: {
                            viewModel.deleteLogs()
                            viewModel.isDeleteDialogDisplayed = false
                        }
                    )
                )
            }
        }
        .fileExporter(
            isPresented: $viewModel.isFileExporterPresented,
            document: TextFile(lineArrray: viewModel.logLines),
            contentType: .plainText,
            defaultFilename: Constants.logFileName.rawValue
        ) { _ in }
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
                .foregroundStyle(NymColor.sysOnSurface)
                .frame(width: 24, height: 24)
                .padding(8)

            Text(title)
                .foregroundStyle(NymColor.sysOnSurface)
                .textStyle(.LabelLegacy.Medium.primary)
        }
        .contentShape(Rectangle())
    }

    @ViewBuilder
    func exportButton() -> some View {
        if let url = viewModel.logFileURL() {
#if os(iOS)
                ShareLink(item: url) {
                    button(systemImageName: "square.and.arrow.up", title: viewModel.exportLocalizedString)
                }
                .disabled(viewModel.logLines.isEmpty)
                .simultaneousGesture(
                    TapGesture().onEnded { viewModel.impactGenerator.impact() }
                )
#endif
#if os(macOS)
            button(systemImageName: "square.and.arrow.up", title: viewModel.exportLocalizedString)
                .onTapGesture {
                    guard !viewModel.logLines.isEmpty else { return }
                    viewModel.isFileExporterPresented.toggle()
                }
#endif
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
    }

    func buttonsSection() -> some View {
        HStack {
            Spacer()
            if #available(iOS 17.0, *), #available(macOS 14.0, *) {
                exportButton()
                Spacer()
                deleteButton()
            } else {
                exportButton()
                Spacer()
                deleteButton()
            }
            Spacer()
        }
        .background {
            NymColor.navigationBarBackground
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
        ScrollViewReader { proxy in
            ScrollView(.vertical) {
                LazyVStack(alignment: .leading) {
                    ForEach(viewModel.logLines.indices, id: \.self) { index in
                        Text(viewModel.logLines[index])
                            .id(index)
                            .onTapGesture(count: 2) {
                                viewModel.copyToPasteboard(index: index)
                            }
                    }
                }
                .padding()
            }
            .onAppear {
                withAnimation {
                    proxy.scrollTo(viewModel.lastLogIndex, anchor: .bottom)
                }
            }
            .onChange(of: viewModel.logLines) { _ in
                withAnimation {
                    proxy.scrollTo(viewModel.lastLogIndex, anchor: .bottom)
                }
            }
        }
    }
}
