public struct NymCountry: Codable, Hashable {
    public let name: String
    public let code: String
    public var regions: [String]

    public init(name: String, code: String, regions: [String]) {
        self.name = name
        self.code = code
        self.regions = regions
    }
}
