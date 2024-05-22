public struct Country: Hashable {
    public let name: String
    public let code: String

    public init(name: String, code: String) {
        self.name = name
        self.code = code
    }
}
