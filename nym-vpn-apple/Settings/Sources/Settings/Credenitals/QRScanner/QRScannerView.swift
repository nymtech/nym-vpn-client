#if os(iOS)
import SwiftUI
import VisionKit
import Theme
import UIComponents

public struct QRScannerView: View {
    @StateObject var viewModel: QRScannerViewModel

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            if viewModel.isScannerAvailable {
                DataScannerRepresentable(
                    shouldStartScanning: $viewModel.isDisplayed,
                    scannedText: $viewModel.scannedText,
                    dataToScanFor: [.barcode(symbologies: [.qr])]
                )
            } else {
                noCameraAccess()
            }
            bottomView()
        }
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onAppear {
            viewModel.hideKeyboard()
        }
    }
}

private extension QRScannerView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.titleLocalizedString,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.hideScanner() })
        )
    }

    @ViewBuilder
    func bottomView() -> some View {
        VStack {
            HStack {
                Spacer()

                Text(viewModel.subtitleLolizedString)
                    .textStyle(.Body.Medium.regular)

                Spacer()
            }
            .padding(16)
            Spacer()
        }
        .frame(height: 120)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }

    @ViewBuilder
    func noCameraAccess() -> some View {
        VStack {
            Spacer()
            Text(viewModel.noCameraPermissionLocalizedString)
                .padding(.horizontal, 16)

            GenericButton(title: viewModel.openSettingsLocalizedString)
                .padding(16)
                .onTapGesture {
                    viewModel.openSettings()
                }
            Spacer()
        }
    }
}
#endif
