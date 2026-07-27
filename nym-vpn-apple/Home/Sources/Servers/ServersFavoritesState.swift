import SwiftUI
import ConnectionTypes
import GatewayManager
import Theme
import UIComponents

/// Filter tabs shown above the gateways list (Figma: Favorites / Recent / All servers).
enum ServerFilter: CaseIterable {
    case favorites
    case recent
    case allServers

    var localizedTitle: String {
        switch self {
        case .favorites:
            "gatewaysView.filter.favorites".localizedString
        case .recent:
            "gatewaysView.filter.recent".localizedString
        case .allServers:
            "gatewaysView.filter.allServers".localizedString
        }
    }

    var systemImageName: String {
        switch self {
        case .favorites:
            "star"
        case .recent:
            "clock"
        case .allServers:
            "list.bullet"
        }
    }
}

/// Servers-list favorites, scoped to the hop the list was opened for.
/// Storage is core's `favorites.json` behind `GatewayManager` — this only owns the
/// selected filter tab and routes reads/writes to the entry or exit list.
@MainActor final class ServersFavoritesState: ObservableObject {
    @Published var filter: ServerFilter = .allServers

    private let hopType: HopType
    private let gatewayManager: GatewayManager

    init(hopType: HopType, gatewayManager: GatewayManager) {
        self.hopType = hopType
        self.gatewayManager = gatewayManager
    }

    var favorites: [ServerFavorite] {
        switch hopType {
        case .entry:
            gatewayManager.entryFavorites
        case .exit:
            gatewayManager.exitFavorites
        }
    }

    func isFavorite(_ favorite: ServerFavorite) -> Bool {
        favorites.contains(favorite)
    }

    func toggleFavorite(_ favorite: ServerFavorite) {
        let isFavorite = !isFavorite(favorite)
        Task {
            switch hopType {
            case .entry:
                await gatewayManager.setEntryFavorite(favorite, isFavorite: isFavorite)
            case .exit:
                await gatewayManager.setExitFavorite(favorite, isFavorite: isFavorite)
            }
        }
    }
}

/// Star toggle used on country and gateway rows.
struct FavoriteStarButton: View {
    let isFavorite: Bool
    let action: () -> Void

    @Environment(\.accessibilityVoiceOverEnabled)
    private var voiceOverEnabled

    var body: some View {
        Button(action: action) {
            Image(systemName: isFavorite ? "star.fill" : "star")
                .font(.system(size: 15, weight: .regular))
                .foregroundStyle(isFavorite ? Color.Nym.primary : Color.Nym.textTertiary)
                .frame(width: 24, height: 24)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .focusEffectDisabled(!voiceOverEnabled)
#if os(macOS)
        .focusable(voiceOverEnabled)
#endif
        .accessibilityLabel(
            isFavorite
            ? "gatewaysView.favorite.remove".localizedString
            : "gatewaysView.favorite.add".localizedString
        )
    }
}
