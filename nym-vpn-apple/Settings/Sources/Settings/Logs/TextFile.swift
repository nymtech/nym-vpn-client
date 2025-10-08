import SwiftUI
import UniformTypeIdentifiers

struct TextFile: FileDocument {
    // Make this immutable or computed to be concurrency-safe
    static let readableContentTypes: [UTType] = [.plainText]

    var text = ""

    init(lineArray: [String]) {
        text = lineArray.joined(separator: "\n")
    }

    init(configuration: ReadConfiguration) throws {
        if let data = configuration.file.regularFileContents {
            text = String(bytes: data, encoding: .utf8) ?? ""
        } else {
            text = ""
        }
    }

    func fileWrapper(configuration: WriteConfiguration) throws -> FileWrapper {
        FileWrapper(regularFileWithContents: Data(text.utf8))
    }
}
