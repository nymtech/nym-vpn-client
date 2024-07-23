#if os(iOS)
import SwiftUI
import VisionKit

public struct DataScannerRepresentable: UIViewControllerRepresentable {
    @Binding var shouldStartScanning: Bool
    @Binding var scannedText: String

    var dataToScanFor: Set<DataScannerViewController.RecognizedDataType>

    public class Coordinator: NSObject, DataScannerViewControllerDelegate {
        var parent: DataScannerRepresentable

        init(_ parent: DataScannerRepresentable) {
            self.parent = parent
        }

        public func dataScanner(_ dataScanner: DataScannerViewController, didTapOn item: RecognizedItem) {
            switch item {
            case .barcode(let barcode):
                guard let qrString = barcode.payloadStringValue else { return }
                parent.scannedText = qrString
            default:
                break
            }
        }

        public func dataScanner(
            _ dataScanner: DataScannerViewController,
            didAdd addedItems: [RecognizedItem],
            allItems: [RecognizedItem]
        ) {
            allItems.forEach { item in
                switch item {
                case .barcode(let barcode):
                    guard let qrString = barcode.payloadStringValue  else { return }
                    parent.scannedText = qrString
                default:
                    break
                }
            }
        }
    }

    public func makeUIViewController(context: Context) -> DataScannerViewController {
        let dataScannerVC = DataScannerViewController(
            recognizedDataTypes: dataToScanFor,
            qualityLevel: .accurate,
            recognizesMultipleItems: false,
            isHighFrameRateTrackingEnabled: true,
            isPinchToZoomEnabled: true,
            isGuidanceEnabled: true,
            isHighlightingEnabled: true
        )

        dataScannerVC.delegate = context.coordinator

        return dataScannerVC
    }

    public func updateUIViewController(_ uiViewController: DataScannerViewController, context: Context) {
        if shouldStartScanning {
            try? uiViewController.startScanning()
        } else {
            uiViewController.stopScanning()
        }
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }
}
#endif
