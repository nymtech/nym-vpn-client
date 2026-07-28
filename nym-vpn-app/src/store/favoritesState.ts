import { create } from 'zustand';
import { Favorite, Favorites } from '../types/tauri';
import { NodeHop } from '../types/util';
import { favoriteKey } from '../types/favorites';

type FavoritesStore = {
  entry: Favorite[];
  exit: Favorite[];
  hydrate: (favorites: Favorites) => void;
  add: (hop: NodeHop, favorite: Favorite) => void;
  remove: (hop: NodeHop, favorite: Favorite) => void;
};

export const useFavoritesStore = create<FavoritesStore>((set, get) => ({
  entry: [],
  exit: [],

  hydrate: ({ entry, exit }) => set({ entry, exit }),

  add: (hop, favorite) => {
    const key = favoriteKey(favorite);
    if (get()[hop].some((f) => favoriteKey(f) === key)) return;
    set((s) => ({ [hop]: [...s[hop], favorite] }));
  },

  remove: (hop, favorite) => {
    const key = favoriteKey(favorite);
    set((s) => ({ [hop]: s[hop].filter((f) => favoriteKey(f) !== key) }));
  },
}));

export const useFavorites = (hop: NodeHop) => useFavoritesStore((s) => s[hop]);
