public enum LogFileType: String, CaseIterable {
#if os(iOS)
    case app
    case tunnel
#elseif os(macOS)
    case app
#endif
}
