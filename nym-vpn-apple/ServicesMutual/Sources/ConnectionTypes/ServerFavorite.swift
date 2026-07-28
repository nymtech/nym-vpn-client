import Foundation

/// A favorited server target, persisted by core in `favorites.json`.
/// Mirrors `nym_vpn_lib_types::FavoriteSelector`; entry and exit are kept as separate lists.
public enum ServerFavorite: Codable, Hashable, Sendable {
    case gateway(String)
    case country(String)
    case region(String)
}
