#if os(iOS)
import SwiftUI
import VisionKit
import ExternalLinkManager
import KeyboardManager
import Theme

import AVFoundation

public final class QRScannerViewModel: ObservableObject {
    private let externalLinkManager: ExternalLinkManager
    private let keyboardManager: KeyboardManager

    let titleLocalizedString = "addCredentials.qrScanner".localizedString
    let subtitleLolizedString = "addCredentials.scanQrCode".localizedString
    let noCameraPermissionLocalizedString = "addCredentials.qrScanner.cameraNotAvailable".localizedString
    let openSettingsLocalizedString = "addCredentials.qrScanner.openSettings".localizedString

    @MainActor @Binding var isDisplayed: Bool
    @Binding var scannedText: String
    @Published var isScanning: Bool
    @Published var isAuthorized = false

    @MainActor var isScannerAvailable: Bool {
        DataScannerViewController.isSupported && DataScannerViewController.isAvailable && isAuthorized
    }

    init(
        isDisplayed: Binding<Bool>,
        scannedText: Binding<String>,
        externalLinkManager: ExternalLinkManager,
        keyboardManager: KeyboardManager
    ) {
        _isDisplayed = isDisplayed
        _scannedText = scannedText
        self.externalLinkManager = externalLinkManager
        self.keyboardManager = keyboardManager
        self.isScanning = false

        checkForPermissions()
    }

    @MainActor func hideScanner() {
        isDisplayed = false
    }

    func hideKeyboard() {
        keyboardManager.hideKeyboard()
    }

    func openSettings() {
        try? externalLinkManager.openExternalURL(urlString: UIApplication.openSettingsURLString)
    }
}

extension QRScannerViewModel {
    func checkForPermissions() {
        switch AVCaptureDevice.authorizationStatus(for: .video) {
        case .authorized:
            isAuthorized = true
            isScanning = true
        case .notDetermined:
            askForCameraPermission()
        default:
            break
        }
    }

    func askForCameraPermission() {
        AVCaptureDevice.requestAccess(for: .video, completionHandler: { [weak self] granted in
            DispatchQueue.main.async {
                if !granted {
                    self?.isScanning = false
                    self?.isAuthorized = false
                } else {
                    self?.isScanning = true
                    self?.isAuthorized = true
                }
            }
        })
    }
}
#endif
