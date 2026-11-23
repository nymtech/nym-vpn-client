import CoreImage
import CoreImage.CIFilterBuiltins
import SwiftUI

/// Black on white, as scanners need the contrast.
public struct QRCodeImage: View {
    private let image: CGImage?
    private let accessibilityLabel: String

    public init(content: String, accessibilityLabel: String) {
        self.image = QRCodeImage.encode(content)
        self.accessibilityLabel = accessibilityLabel
    }

    public var body: some View {
        Group {
            if let image {
                Image(decorative: image, scale: 1)
                    .interpolation(.none)
                    .resizable()
                    .scaledToFit()
            } else {
                Color.white
            }
        }
        .aspectRatio(1, contentMode: .fit)
        .padding(Constants.padding)
        .background(Color.white, in: RoundedRectangle(cornerRadius: Constants.cornerRadius))
        .accessibilityElement()
        .accessibilityLabel(accessibilityLabel)
        .accessibilityAddTraits(.isImage)
    }

    static func encode(_ content: String) -> CGImage? {
        let filter = CIFilter.qrCodeGenerator()
        filter.message = Data(content.utf8)
        filter.correctionLevel = "M"
        guard let output = filter.outputImage else { return nil }
        return CIContext().createCGImage(output, from: output.extent)
    }

    private enum Constants {
        static let padding: CGFloat = 12
        static let cornerRadius: CGFloat = 12
    }
}
