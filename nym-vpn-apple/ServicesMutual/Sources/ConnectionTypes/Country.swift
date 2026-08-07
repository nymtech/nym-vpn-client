public struct NymCountry: Codable, Hashable {
    public let name: String
    public let code: String
    public var regions: [Region]

    public init(name: String, code: String, regions: [Region]) {
        self.name = name
        self.code = code
        self.regions = regions
    }

    public struct Region: Codable, Hashable {
        public let name: String
        public var cities: [String]

        public init(name: String, cities: [String]) {
            self.name = name
            self.cities = cities
        }
    }
}
