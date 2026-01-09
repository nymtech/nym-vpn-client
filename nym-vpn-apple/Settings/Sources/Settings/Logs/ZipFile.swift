import SwiftUI
import UniformTypeIdentifiers

struct ZipFile: FileDocument {
    static let readableContentTypes: [UTType] = [.zip]

    var url: URL

    init(url: URL) {
        self.url = url
    }

    init(configuration: ReadConfiguration) throws {
        throw CocoaError(.fileReadUnknown)
    }

    func fileWrapper(configuration: WriteConfiguration) throws -> FileWrapper {
        try FileWrapper(url: url, options: .immediate)
    }
}
