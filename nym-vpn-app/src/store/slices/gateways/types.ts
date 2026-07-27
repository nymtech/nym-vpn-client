import {
  AppError,
  FavoriteKind,
  Favorites,
  Gateway,
  GatewayType,
  GatewaysByCountry,
  NodeHop,
} from '../../../types';

export type GatewaysSlice = GatewaysState & {
  fetchGateways: (nodeType: GatewayType) => Promise<void>;
  lookupGw: (
    id: string,
    type: 'entry' | 'exit',
    countryCode?: string,
  ) => Gateway | null;
  // Load persisted favorites from the core FavoritesManager into memory
  loadFavorites: () => Promise<void>;
  // Toggle a favorite for a hop and persist via the core FavoritesManager
  toggleFavorite: (
    hop: NodeHop,
    kind: FavoriteKind,
    value: string,
  ) => Promise<void>;
  // Whether an item is favorited for the given hop
  isFavorite: (hop: NodeHop, kind: FavoriteKind, value: string) => boolean;
};

export type GatewaysState = {
  mxEntry: GatewaysByCountry[];
  mxExit: GatewaysByCountry[];
  wg: GatewaysByCountry[];
  mxEntryLoading: boolean;
  mxExitLoading: boolean;
  wgLoading: boolean;
  mxEntryError: AppError | null;
  mxExitError: AppError | null;
  wgError: AppError | null;
  // Favorites are partitioned per hop (entry/exit), backed by the core
  // FavoritesManager. Not split by tunnel type.
  favorites: Favorites;
};
