public enum LogFileType: String, CaseIterable {
#if os(iOS)
    case app
    case tunnel
    case library
#elseif os(macOS)
    case app
    case daemon
#endif
}
