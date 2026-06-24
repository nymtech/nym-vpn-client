import Foundation

enum DnsCustomListLayoutPolicy: Equatable, Sendable {
    /// Zero-row nested `List` still mounts UITableView chrome and can draw a vertical separator through siblings.
    static func shouldRenderReorderableList(entryCount: Int) -> Bool {
        entryCount > 0
    }
}
