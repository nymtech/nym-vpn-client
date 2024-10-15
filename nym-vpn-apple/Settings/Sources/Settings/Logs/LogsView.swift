import SwiftUI
import Constants
import Theme
import UIComponents

public struct LogsView: View {
    @ObservedObject private var viewModel: LogsViewModel

    public init(viewModel: LogsViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack {
            navbar()

            VStack {
                if !viewModel.logs.isEmpty {
                    ScrollViewReader { proxy in
                        ScrollView(.vertical) {
                            Text(viewModel.logs)
                                .id("log")
                                .padding()
                        }
                        .onAppear {
                            withAnimation {
                                proxy.scrollTo("log", anchor: .bottom)
                            }
                        }
                    }
                } else {
                    VStack {
                        Spacer()
                        Text(viewModel.noLogsLocalizedString)
                        Spacer()
                    }
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
            document: TextFile(initialText: viewModel.logs),
            contentType: .plainText,
            defaultFilename: Constants.logFileName.rawValue
        ) { _ in }
    }
}

private extension LogsView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
        .padding(0)
    }

    @ViewBuilder
    func button(systemImageName: String, title: String) -> some View {
        VStack {
            Image(systemName: systemImageName)
                .foregroundStyle(NymColor.sysOnSurface)
                .frame(width: 24, height: 24)
                .padding(8)

            Text(title)
                .foregroundStyle(NymColor.sysOnSurface)
                .textStyle(.Label.Medium.primary)
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
                .disabled(viewModel.logs.isEmpty)
                .simultaneousGesture(
                    TapGesture().onEnded { viewModel.impactGenerator.impact() }
                )
#endif
#if os(macOS)
            button(systemImageName: "square.and.arrow.up", title: viewModel.exportLocalizedString)
                .onTapGesture {
                    guard !viewModel.logs.isEmpty else { return }
                    viewModel.isFileExporterPresented.toggle()
                }
#endif
        }
    }

    @ViewBuilder
    func deleteButton() -> some View {
        button(systemImageName: "trash", title: viewModel.deleteLocalizedString)
            .disabled(viewModel.logs.isEmpty)
            .onTapGesture {
#if os(iOS)
                viewModel.impactGenerator.impact()
#endif
                if !viewModel.logs.isEmpty {
                    viewModel.isDeleteDialogDisplayed.toggle()
                }
            }
    }

    @ViewBuilder
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
        Picker("", selection: $viewModel.currentLogFileType) {
            ForEach(viewModel.logFileTypes, id: \.self) {
                Text($0.rawValue.capitalized.localizedString)
            }
        }
        .pickerStyle(.segmented)
        .padding(16)
    }
}
