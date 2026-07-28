import Foundation
import ConnectionTypes
import PathManager
#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import NymVPNRpc
#endif

// Favorites are a client-side file (`favorites.json` in the group data folder), not daemon
// state — both platforms drive `FavoritesController` directly, iOS through NymVPNLib and
// macOS through NymVPNRpc.
extension GatewayWorker {
    func fetchFavorites() async throws -> (entry: [ServerFavorite], exit: [ServerFavorite]) {
        let selectors = try await controller().getFavorites()
        return (selectors.entry.map { ServerFavorite(selector: $0) },
                selectors.exit.map { ServerFavorite(selector: $0) })
    }

    func setEntryFavorite(_ favorite: ServerFavorite, isFavorite: Bool) async throws {
        let controller = try await controller()
        if isFavorite {
            try await controller.addFavoriteEntry(selector: favorite.selector)
        } else {
            try await controller.removeFavoriteEntry(selector: favorite.selector)
        }
    }

    func setExitFavorite(_ favorite: ServerFavorite, isFavorite: Bool) async throws {
        let controller = try await controller()
        if isFavorite {
            try await controller.addFavoriteExit(selector: favorite.selector)
        } else {
            try await controller.removeFavoriteExit(selector: favorite.selector)
        }
    }

    /// Built per call: the controller caches the file contents at init, so a fresh one is
    /// the cheapest way to stay correct across environment switches and external writes.
    /// Data dir is the group data folder, matching `nym-vpnc`'s `app_data_dir()` — favorites
    /// are not network-scoped.
    private func controller() async throws -> FavoritesController {
        let dataDir = try PathManager.dataFolderURL().path(percentEncoded: false)
        return await FavoritesController(dataDir: dataDir)
    }
}

private extension ServerFavorite {
    var selector: FavoriteSelector {
        switch self {
        case let .gateway(identity):
            .gateway(identity: identity)
        case let .country(countryCode):
            .country(twoLetterIsoCountryCode: countryCode)
        case let .region(region):
            .region(region: region)
        }
    }

    init(selector: FavoriteSelector) {
        switch selector {
        case let .gateway(identity):
            self = .gateway(identity)
        case let .country(twoLetterIsoCountryCode):
            self = .country(twoLetterIsoCountryCode)
        case let .region(region):
            self = .region(region)
        }
    }
}
