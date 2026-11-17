#if os(iOS)
import AVFoundation
import SwiftUI
import VisionKit

struct QRCameraPreview: UIViewControllerRepresentable {
    let isScanning: Bool
    let onCodeScan: (String) -> Void
    let onCameraError: () -> Void

    static var isSupported: Bool {
        DataScannerViewController.isSupported
    }

    func makeUIViewController(context: Context) -> DataScannerViewController {
        let scanner = DataScannerViewController(
            recognizedDataTypes: [.barcode(symbologies: [.qr])],
            qualityLevel: .balanced,
            recognizesMultipleItems: false,
            isHighFrameRateTrackingEnabled: false,
            isPinchToZoomEnabled: true,
            isGuidanceEnabled: false,
            isHighlightingEnabled: false
        )
        scanner.delegate = context.coordinator
        return scanner
    }

    func updateUIViewController(_ scanner: DataScannerViewController, context: Context) {
        context.coordinator.parent = self
        if isScanning, !scanner.isScanning {
            do {
                try scanner.startScanning()
            } catch {
                reportCameraError()
            }
        } else if !isScanning, scanner.isScanning {
            scanner.stopScanning()
        }
    }

    static func dismantleUIViewController(_ scanner: DataScannerViewController, coordinator: Coordinator) {
        scanner.stopScanning()
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    fileprivate func reportCameraError() {
        Task { @MainActor in onCameraError() }
    }

    final class Coordinator: NSObject, DataScannerViewControllerDelegate {
        var parent: QRCameraPreview

        init(parent: QRCameraPreview) {
            self.parent = parent
        }

        func dataScanner(
            _ dataScanner: DataScannerViewController,
            didAdd addedItems: [RecognizedItem],
            allItems: [RecognizedItem]
        ) {
            report(addedItems)
        }

        // A code that stays in view keeps arriving here, so it can be retried.
        func dataScanner(
            _ dataScanner: DataScannerViewController,
            didUpdate updatedItems: [RecognizedItem],
            allItems: [RecognizedItem]
        ) {
            report(updatedItems)
        }

        func dataScanner(
            _ dataScanner: DataScannerViewController,
            becameUnavailableWithError error: DataScannerViewController.ScanningUnavailable
        ) {
            parent.reportCameraError()
        }

        private func report(_ items: [RecognizedItem]) {
            for item in items {
                guard case let .barcode(barcode) = item, let payload = barcode.payloadStringValue else { continue }
                parent.onCodeScan(payload)
                return
            }
        }
    }
}

enum QRTorch {
    static var isAvailable: Bool {
        AVCaptureDevice.default(for: .video)?.hasTorch == true
    }

    static func set(enabled: Bool) {
        guard let device = AVCaptureDevice.default(for: .video), device.hasTorch else { return }
        do {
            try device.lockForConfiguration()
            device.torchMode = enabled ? .on : .off
            device.unlockForConfiguration()
        } catch {
            // The torch is optional.
        }
    }
}
#endif
